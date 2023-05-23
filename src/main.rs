use poem::{
    listener::TcpListener, web::Data, EndpointExt,
    Result, Route, Server,
};
use poem_openapi::{param::Query, payload::PlainText, OpenApi, OpenApiService};
use poem_openapi::{
    payload::Json,
    ApiResponse, Object, Tags,
};

use serde::{Deserialize, Serialize};

extern crate config;
extern crate serde;
mod settings;

use settings::Settings;

use redis::{FromRedisValue, Value};

use redis_derive::{FromRedisValue, ToRedisArgs};
use ulid::Ulid;

#[derive(Debug, Deserialize, Serialize)]
struct Edge {
    source: String,
    target: String,
    rank: f64,
    year: Option<i64>,
}



#[derive(Tags)]
enum ApiTags {
    /// Operations about articles
    Article,
    SearchQuery
}

/// Create article schema
#[derive(Debug, Object, FromRedisValue, ToRedisArgs)]
struct Article {
    id:Option<String>,
    #[oai(validator(max_length = 254))]
    stub: Option<String>,
    title: String,
    url: String,
    body: String,
    description: Option<String>,
    tags: Option<Vec<String>>

}

#[derive(Debug, Object)]
struct SearchQuery{
    search_term:String,
    skip:usize,
    limit:usize,
    role: Option<String>,
}

//  FT.CREATE ArticleIdx ON HASH PREFIX 1 article: SCHEMA title TEXT WEIGHT 5.0 body TEXT url TEXT


#[derive(Object,Serialize, Deserialize, Debug)]
pub struct RedisearchResult {
        id:String,
        stub: Option<String>,
        title: String,
        url: String,
        body: String,
        description: Option<String>,
        tags: Option<Vec<String>>
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
        println!(" Values - - - {:?}",values);
        for i in 0..values.len() {
            match values[i].as_str() {
                "id" => id= values[i + 1].clone(),
                "title" => title = values[i + 1].clone(),
                "stub" => stub = values[i + 1].clone(),
                "url" => url= values[i + 1].clone(),
                "body" => body = values[i + 1].clone(),
                "description" => description = values[i + 1].clone(),
                "tags" => tags = values[i + 1].clone().split(',').map(|s| s.trim().to_string()).collect(),
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
    async fn create_article(&self, settings: Data<&Settings>, article: Json<Article>) -> CreateArticleResponse {
        
        let id  = Ulid::new().to_string();
        println!("{:?}",article);

        let url = settings.redis_url.clone();
        let client = redis::Client::open(url).unwrap();
        let mut con = client.get_connection().unwrap();
        let _: () = redis::cmd("PING").query(&mut con).unwrap();
        println!("Aricle {:?}",article);
        // let _: () = con.hset_multiple(
        //     format!("article:{}",id),
        //     &[  ("id", &id),
        //         ("title", &article.title),
        //         ("url", &article.url),
        //         ("body", &article.body)
        //     ],
        // )
        // .unwrap();
        let _: () = redis::cmd("HSET")
        .arg(format!("article:{}",id))
        .arg(&*article)
        .query(&mut con).unwrap();
        CreateArticleResponse::Ok(Json(id))
    }
    // #[oai(path = "/gsearch/", method = "post", tag = "ApiTags::SearchQuery")]
    // async fn graph_search(&self, settings: Data<&Settings>,search_query: Json<SearchQuery>) -> Json<Vec<RedisearchResult>> {
    //     let role = search.role.as_deref().unwrap_or("");
    //     println!("Role {}", role);
    // let automata_url = if role == "Medical" {
    //     "https://s3.eu-west-2.amazonaws.com/assets.thepattern.digital/automata_fresh_semantic.pkl.lzma"
    // } else {
    //     "https://terraphim-automata.s3.eu-west-2.amazonaws.com/automata_cyberattack.lzma"
    // };
    // let automata = load_matcher(automata_url).unwrap();
    // let nodes = match_nodes(&search.search, &automata);
    // println!("Nodes {:?}", nodes);
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
    async fn find_article(&self, settings: Data<&Settings>,search_query: Json<SearchQuery>) -> Json<Vec<RedisearchResult>> {
        let url = settings.redis_url.clone();
        let client = redis::Client::open(url).unwrap();
        let mut con = client.get_connection().unwrap();
        println!("{:#?}",search_query);

        let values: Vec<Value>= redis::cmd("FT.SEARCH").arg("ArticleIdx")
        .arg(&search_query.search_term).arg("LIMIT")
        .arg(search_query.skip)
        .arg(search_query.limit).query(&mut con).unwrap();
        println!("Output of scan");
        println!("{:#?}",values);
        let array_value = redis::Value::Bulk(values);
        let results = parse_redisearch_response(&array_value);
        Json(results)
    }
    
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let settings = Settings::new().unwrap();
    println!("{:?}",settings);
    let bind_addr = settings.server_url.clone();
    let api_endpoint = settings.api_endpoint.clone();
    let api_service =
        OpenApiService::new(Api, "Hello World", "1.0").server(api_endpoint);
    let ui = api_service.swagger_ui();
    let spec = api_service.spec();
    let route = Route::new()
    .nest("/api", api_service)
    .nest("/ui", ui)
    .at("/spec", poem::endpoint::make_sync(move |_| spec.clone()))
    // .with(Cors::new())
    .data(settings);
    
    Server::new(TcpListener::bind(bind_addr))
    .run(route)
    .await?;

    Ok(())
}