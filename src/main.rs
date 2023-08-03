use poem::{listener::TcpListener, web::Data, EndpointExt, Result, Route, Server};
use poem_openapi::{param::Query, payload::PlainText, OpenApi, OpenApiService};
use poem_openapi::{payload::Json, ApiResponse, Object, Tags};
use serde::{Deserialize, Serialize};
use std::error::Error;
use itertools::Itertools;
extern crate config;
extern crate serde;
mod settings;

use settings::Settings;
use regex::Regex;

#[macro_use]
extern crate lazy_static;
lazy_static! {
    static ref RE: Regex = Regex::new(r"[?!|]\s+").unwrap();
}

use redis::{FromRedisValue, Value, Commands};
use redis_derive::{FromRedisValue, ToRedisArgs};
use ulid::Ulid;

use terraphim_automata::{find_matches, load_automata, Dictionary, Matched};
use terraphim_pipeline::split_paragraphs;
mod graph_search;
use graph_search::{get_edges, match_nodes, Edge};

#[derive(Tags)]
enum ApiTags {
    /// Operations about articles
    Article,
    SearchQuery,
}

/// Create article schema
#[derive(Deserialize, Serialize, Debug, Object, FromRedisValue, ToRedisArgs)]
struct Article {
    id: Option<String>,
    stub: Option<String>,
    title: String,
    url: String,
    body: String,
    description: Option<String>,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Object)]
struct SearchQuery {
    search_term: String,
    skip: usize,
    limit: usize,
    role: Option<String>,
}

//  FT.CREATE ArticleIdx ON HASH PREFIX 1 article: SCHEMA title TEXT WEIGHT 5.0 body TEXT url TEXT

// TODO: check if can be rewritten nice with https://docs.rs/struct-field-names-as-array/latest/struct_field_names_as_array/ or macros
#[derive(Object, Serialize, Deserialize, Debug)]
pub struct RedisearchResult {
    id: String,
    stub: Option<String>,
    title: String,
    url: String,
    body: String,
    description: Option<String>,
    tags: Option<Vec<String>>,
}

impl FromRedisValue for RedisearchResult {
    fn from_redis_value(v: &Value) -> redis::RedisResult<Self> {
        let values: Vec<String> = redis::from_redis_value(v)?;
        let mut id = String::new();
        let mut title = String::new();
        let mut stub = String::new();
        let mut url = String::new();
        let mut body = String::new();
        let mut description = String::new();
        let mut tags = vec![<String>::new()];
        println!(" Values - - - {:?}", values);
        for i in 0..values.len() {
            match values[i].as_str() {
                "id" => id = values[i + 1].clone(),
                "title" => title = values[i + 1].clone(),
                "stub" => stub = values[i + 1].clone(),
                "url" => url = values[i + 1].clone(),
                "body" => body = values[i + 1].clone(),
                "description" => description = values[i + 1].clone(),
                "tags" => {
                    tags = values[i + 1]
                        .clone()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect()
                }
                _ => continue,
            }
        }

        Ok(RedisearchResult {
            id,
            stub: stub.parse().ok(),
            title,
            url,
            body,
            description: description.parse().ok(),
            tags: Some(tags),
        })
    }
}

pub fn parse_redisearch_response(response: &Value) -> Vec<RedisearchResult> {
    match response {
        Value::Bulk(array) => {
            let mut results = Vec::new();
            let n = array.len();

            for item in array.iter().take(n).skip(1) {
                if let Value::Bulk(ref bulk) = item {
                    if let Ok(result) =
                        RedisearchResult::from_redis_value(&Value::Bulk(bulk.clone()))
                    {
                        results.push(result);
                    }
                }
            }

            results
        }
        _ => vec![],
    }
}

#[derive(ApiResponse)]
enum CreateArticleResponse {
    /// Returns when the article is successfully created.
    #[oai(status = 200)]
    Ok(Json<String>),
}

#[derive(ApiResponse)]
enum FindArticleResponse {
    /// Return the specified user.
    #[oai(status = 200)]
    Ok(Json<Article>),
    /// Return when the specified user is not found.
    #[oai(status = 404)]
    NotFound,
}
#[derive(ApiResponse)]
enum DeleteArticleResponse {
    /// Returns when the user is successfully deleted.
    #[oai(status = 200)]
    Ok,
    /// Return when the specified user is not found.
    #[oai(status = 404)]
    NotFound,
}

#[derive(ApiResponse)]
enum UpdateArticleResponse {
    /// Returns when the user is successfully updated.
    #[oai(status = 200)]
    Ok,
    /// Return when the specified user is not found.
    #[oai(status = 404)]
    NotFound,
}

async fn parse_article(article: &Article, id:&str,role: &str, automata_url: &str, con: &mut redis::Connection) -> redis::RedisResult<()>{
    
    for sentence in split_paragraphs(&article.body) {
        println!("{}", sentence);
        let shard_id= "{06S}";
        // for each role run
        // println!("Role {}", role);
        // let automata_url = "./crates/terraphim_automata/data/output.csv.gz";
        let automata = load_automata(automata_url).unwrap();
        let matched_ents = find_matches(sentence, automata, false).expect("Failed to find matches");
        println!("Nodes {:?}", matched_ents);
        
        for pair in matched_ents.into_iter().combinations(2) {
            println!("Pair {:?}", pair);
            let source_entity_id = pair[0].id.clone();
            let destination_entity_id = pair[1].id.clone();
            let source_canonical_name = pair[0].term.clone();
            let destination_canonical_name = pair[1].term.clone();
            
            let _: () = redis::cmd("XADD")
            .arg(format!("edges_matched_{role}_{shard_id}"))
            .arg("*")
            .arg("source")
            .arg(source_entity_id.clone())
            .arg("destination")
            .arg(destination_entity_id.clone())
            .arg("source_name")
            .arg(source_canonical_name)
            .arg("destination_name")
            .arg(destination_canonical_name)
            .arg("rank")
            .arg(1)
            .arg("year")
            .arg(2023)
            .execute(con);
            let _: () = redis::cmd("ZINCRBY")
            .arg(format!("edges_scored:{}:{}",source_entity_id,destination_entity_id))
            .arg(1)
            .arg(&id)
            .execute(con);

        }
    }
    Ok(())
}

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/hello", method = "get")]
    async fn index(&self, name: Query<Option<String>>) -> PlainText<String> {
        match name.0 {
            Some(name) => PlainText(format!("hello, {name}!")),
            None => PlainText("hello!".to_string()),
        }
    }
    #[oai(path = "/articles", method = "post", tag = "ApiTags::Article")]
    async fn create_article(
        &self,
        settings: Data<&Settings>,
        article: Json<Article>,
    ) -> CreateArticleResponse {
        let id = Ulid::new().to_string();

        let url = settings.redis_url.clone();
        let client = redis::Client::open(url).unwrap();
        let mut con = client.get_connection().unwrap();
        // println!("Aricle {:?}",article);
        let _: () = redis::cmd("HSET")
            .arg(format!("article:{}", id))
            .arg(&*article)
            .query(&mut con)
            .unwrap();
        let role= "project-manager";
        let automata_url = "./test-data/term_to_id.json";
        let _ = parse_article(&article, &id, role, automata_url, &mut con).await;
        // let nodes = vec![settings.redis_cluster_url.clone(),"redis://127.0.0.1:30002/".to_string()];
        // let cluster_client = ClusterClient::new(nodes).unwrap();
        // let mut cluster_connection = cluster_client.get_async_connection().await.unwrap();
        // let body = article.body.split('\n').collect::<Vec<&str>>().join(" ");
        // let _: () = con.set(format!("paragraphs:{}",&id),body).await.unwrap();
        // split paragraph by stentences

        // let _: () = redis::cmd("SET")
        //     .arg(format!("paragraphs:{}", &id))
        //     .arg(body)
        //     .query(&mut con)
        //     .unwrap();

        CreateArticleResponse::Ok(Json(id))
    }

    #[oai(path = "/rsearch/", method = "post", tag = "ApiTags::SearchQuery")]
    async fn graph_search(
        &self,
        settings: Data<&Settings>,
        search_query: Json<SearchQuery>,
    ) -> Json<Vec<String>> {
        println!("{:#?}", search_query);
        let role = search_query.role.as_deref().unwrap_or("");
        println!("Role {}", role);
        let automata_url = "./test-data/term_to_id.json";
        let automata = load_automata(automata_url).unwrap();
        let nodes = match_nodes(&search_query.search_term, automata);
        println!("Nodes {:?}", nodes);
        let links = get_edges(&settings, &nodes, None, 50);
        println!("Links {:?}", links);

        Json(nodes)
    }

    //
    // let links = get_edges(&nodes, Some(50), None, None).unwrap();
    // println!("Links {:?}", links);
    // let mut result_table = Vec::new();
    // let mut article_set = std::collections::HashSet::new();
    // let url = settings.redis_url.clone();
    // let client = redis::Client::open(url).unwrap();
    // let mut con = client.get_connection().unwrap();
    // for each_record in links.iter().take(50) {
    //     let edge_query = format!("{}:{}", each_record.source, each_record.target);
    //     println!("Edge query {}", edge_query);
    //     let edge_scored: Vec<String> = con.zrangebyscore_withscores(
    //         format!("edges_scored:{}", edge_query),
    //         "-inf",
    //         "+inf",
    //         (0, 5),
    //     ).unwrap().into_iter().map(|(k, _)| k).collect();
    //     println!("Edge scored {:?}", edge_scored);
    //     for sentence_key in edge_scored {
    //         let mut parts = sentence_key.split(':');
    //         let article_id = parts.nth(1).unwrap();
    //         if article_set.insert(article_id.to_owned()) {
    //             let title: String = con.hget(format!("article_id:{}", article_id), "title").unwrap();
    //             let hash_tag = parts.last().unwrap();
    //             result_table.push(SearchResult {
    //                 title,
    //                 pk: hash_tag.to_owned(),
    //                 url: "".to_owned(),
    //             });
    //         }
    //     }
    // }
    // }

    /// Find article by search term
    #[oai(path = "/search/", method = "post", tag = "ApiTags::SearchQuery")]
    async fn find_article(
        &self,
        settings: Data<&Settings>,
        search_query: Json<SearchQuery>,
    ) -> Json<Vec<RedisearchResult>> {
        let url = settings.redis_url.clone();
        let client = redis::Client::open(url).unwrap();
        let mut con = client.get_connection().unwrap();
        println!("{:#?}", search_query);

        let values: Vec<Value> = redis::cmd("FT.SEARCH")
            .arg("ArticleIdx")
            .arg(&search_query.search_term)
            .arg("LIMIT")
            .arg(search_query.skip)
            .arg(search_query.limit)
            .query(&mut con)
            .unwrap();
        println!("Output of scan");
        println!("{:#?}", values);
        let array_value = redis::Value::Bulk(values);
        let results = parse_redisearch_response(&array_value);
        Json(results)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let settings = Settings::new().unwrap();
    println!("{:?}", settings);
    let bind_addr = settings.server_url.clone();
    let api_endpoint = settings.api_endpoint.clone();
    let api_service = OpenApiService::new(Api, "Hello World", "1.0").server(api_endpoint);
    let ui = api_service.swagger_ui();
    let spec = api_service.spec();
    let route = Route::new()
        .nest("/api", api_service)
        .nest("/doc", ui)
        .at("/spec", poem::endpoint::make_sync(move |_| spec.clone()))
        // .with(Cors::new())
        .data(settings);

    Server::new(TcpListener::bind(bind_addr)).run(route).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use tracing_subscriber::fmt::format;

    use super::*;

    #[test]
    fn test_parse_article() {
        
        use serde_json;
        use std::fs;
        let input = fs::read_to_string("test-data/article.json").unwrap();
        let article: Article = serde_json::from_str(&input).unwrap();
        let role = "project-manager";
        let shard_id = 1;
        let automata_url = "./test-data/term_to_id.json";
        let expected_output = vec![Matched {
            term: "project manager".to_string(),
            id: "project-manager".to_string(),
            nterm: "project management".to_string(),
            pos: Some((0, 14)),
        }];
        let nodes = parse_article(&article, role, automata_url);

        // assert_eq!(parse_article(&article,role, automata_url), expected_output);
    }
}
