use poem::{listener::TcpListener, Route, Server};
use poem_openapi::{param::Query, payload::PlainText, OpenApi, OpenApiService};
use poem_openapi::{
    param::Path,
    payload::Json,
    types::{ParseFromJSON, ToJSON},
    ApiResponse, Object, Tags,
};

use redis::{Commands, Value};
// use redis::{self, FromRedisValue, RedisError, Value};
use redis_derive::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};


use uuid::Uuid;
use ulid::Ulid;

#[derive(Tags)]
enum ApiTags {
    /// Operations about user
    Article,
    SearchQuery
}


//  FIXME: TODO: generic object to connect to Atomic Data Resource

#[derive(Object)]
struct MyResource<T: ParseFromJSON + ToJSON> {
    value: T,
}

/// Create user schema
#[derive(Debug, Object,Serialize, Deserialize, FromRedisValue)]
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
    limit:usize
}

#[derive(Debug, Serialize, Deserialize,FromRedisValue)]
struct SearchResults{
    count:i64,
    results:Vec<Article>
}


//  FT.CREATE ArticleIdx ON HASH PREFIX 1 article: SCHEMA title TEXT WEIGHT 5.0 body TEXT url TEXT

fn string_from_redis_value(v: &Value) -> Option<String> {
    match v {
        Value::Data(d) => String::from_utf8(d.to_vec()).ok(),
        _ => None,
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
    async fn create_article(&self, article: Json<Article>) -> CreateArticleResponse {
        
        let article_id= uuid::Uuid::new_v4().to_string();
        let id  = Ulid::new().to_string();
        println!("{:?}",article);

        let url = "redis://127.0.0.1:6379";
        let client = redis::Client::open(url).unwrap();
        let mut con = client.get_connection().unwrap();
        println!("{:?}",article);
        let _: () = con.hset_multiple(
            format!("article:{}",id),
            &[  ("id", &id.to_string()),
                ("title", &article.title),
                ("url", &article.url),
                ("body", &article.body)
            ],
        )
        .unwrap();
        // let _ = redis::cmd("HSET")
        // .arg(id)
        // .arg(article)
        // .query(&mut con).unwrap();
        CreateArticleResponse::Ok(Json(article_id))
    }
    #[oai(path = "/redisinfo", method = "get")]
    async fn redisinfo(&self, name: Query<Option<String>>) -> PlainText<String> {
        let url = "redis://127.0.0.1:6379";
        let client = redis::Client::open(url).unwrap();
        let mut con = client.get_connection().unwrap();
        do_print_max_entry_limits(&mut con).unwrap();
        match name.0 {
            Some(name) => PlainText(format!("hello, {name}!")),
            None => PlainText("hello!".to_string()),
        }
    }
    /// Find article by search term
    #[oai(path = "/search/", method = "post", tag = "ApiTags::SearchQuery")]
    async fn find_article(&self, searchQuery: Json<SearchQuery>) -> Json<String> {
        let url = "redis://127.0.0.1:6379";
        let client = redis::Client::open(url).unwrap();
        let mut con = client.get_connection().unwrap();
        println!("{:#?}",searchQuery);

        let response: SearchResults= redis::cmd("FT.SEARCH").arg("ArticleIdx")
        .arg(&searchQuery.search_term).arg("LIMIT")
        .arg(&searchQuery.skip)
        .arg(&searchQuery.limit).query(&mut con).unwrap();
        let results:SearchResults= match response {
            _ => vec![],
        };
        println!("{:#?}",results);
        Json("Ok".to_string())  
    }
    
}



use std::collections::HashMap;
use std::result;

fn do_print_max_entry_limits(con: &mut redis::Connection) -> redis::RedisResult<()> {
    // since rust cannot know what format we actually want we need to be
    // explicit here and define the type of our response.  In this case
    // String -> int fits all the items we query for.
    let config: HashMap<String, isize> = redis::cmd("CONFIG")
        .arg("GET")
        .arg("*-max-*-entries")
        .query(con)?;

    println!("Max entry limits:");

    println!(
        "  max-intset:        {}",
        config.get("set-max-intset-entries").unwrap_or(&0)
    );
    println!(
        "  hash-max-ziplist:  {}",
        config.get("hash-max-ziplist-entries").unwrap_or(&0)
    );
    println!(
        "  list-max-ziplist:  {}",
        config.get("list-max-ziplist-entries").unwrap_or(&0)
    );
    println!(
        "  zset-max-ziplist:  {}",
        config.get("zset-max-ziplist-entries").unwrap_or(&0)
    );

    Ok(())
}
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();

    let api_service =
        OpenApiService::new(Api, "Hello World", "1.0").server("http://localhost:3000/api");
    let ui = api_service.swagger_ui();

    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(Route::new().nest("/api", api_service).nest("/", ui))
        .await
}
