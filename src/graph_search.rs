use terraphim_automata::{Dictionary, Matched, load_automata, find_matches};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;


use redis::{FromRedisValue, Value};

use redis_derive::{FromRedisValue, ToRedisArgs};

use redis::Commands;

mod graph_types;
use graph_types::{GraphResultSet, GraphResult};


use crate::settings::{Settings, self};


#[derive(Debug, Deserialize, Serialize, FromRedisValue)]
pub struct Edge {
    e_id: String,
    t_id: String,
    rank: f64,
    year: Option<i64>,
}

pub fn match_nodes(search_string: &str, automata: HashMap<String, Dictionary>) -> Vec<String> {
    let matched_ents = find_matches(search_string, automata, false).expect("Failed to find matches");
    let nodes: HashSet<String> = matched_ents.iter().map(|ent| ent.id.clone()).collect();
    let nodes: Vec<String> = nodes.into_iter().collect();
    // let nodes: Vec<String> = Vec::new();
    // for ent in matched_ents.iter() {
    //     println!("Matched ent {:#?}", ent.id);
    // }
    // println!("Matched ents {:#?}", matched_ents);
    println!("Matched nodes {:#?}", nodes);
    nodes
}

pub fn get_edges(
    settings: &Settings,
    nodes: &[String],
    years: Option<&[&str]>,
    limits: i64,
) -> Vec<Edge> {
    let mut links = Vec::new();
    // let mut nodes_set = std::collections::HashSet::new();
    let mut years_set:Vec<String> = Vec::new();
    let url = settings.redis_url.clone();
    let client = redis::Client::open(url).unwrap();
    let mut con = client.get_connection().unwrap();
    let graph_name = "cord19medical";
    let ids = format!("[{}]", nodes.join(",")).replace("\"", "\'");
    
    // let query = if let Some(years) = years {
    //     let years: Vec<String> = years.iter().map(String::from).collect();
    //     format!(
    //         "WITH {ids} as ids
    //         MATCH (e:entity)-[r]->(t:entity)
    //         WHERE e.id IN ids AND r.year IN {years}
    //         RETURN DISTINCT e.id, t.id, max(r.rank), r.year
    //         ORDER BY r.rank DESC LIMIT {limits}",
    //         ids = nodes, years=years, limits = limits
    //     )
    // } else {
        // query withot years
        // };
        let query =format!("WITH {ids} as ids MATCH (e:entity)-[r]->(t:entity) WHERE e.id IN ids RETURN DISTINCT e.id, t.id, max(r.rank), r.year ORDER BY r.rank DESC LIMIT {limits}", ids = ids, limits = limits);
        println!("Query: {}", query);
        
        let result_set=redis::cmd("GRAPH.QUERY")
            .arg(graph_name)
            .arg(query)
            .query::<redis::Value>(&mut con)
            .unwrap();
    println!("Result set: {:?}", result_set);
    let result_set: GraphResultSet = GraphResultSet::from_redis_value(&result_set).unwrap();
    println!("Result set: {:?}", result_set);
    // let result_vec: Vec<redis::Value> = redis::from_redis_value(&result_set).unwrap();
    // // let edges= RedisearchResult::from_redis_value(&result_set);
    // println!("Result set: {:?}", result_vec.len());
    // println!("Result set 0: {:?}", result_vec[0]);
    // println!("Result set 1: {:?}", result_vec[1]);
    // println!("Result set 2: {:?}", result_vec[2]);

    // let my_struct: MyStruct= redis::from_redis_value(&result_vec[0]).unwrap();
    // println!("Struct: {:?}", my_struct);
    // // for record in result_set {
    //     let source_id = record[0].as_str().unwrap().to_owned();
    //     let target_id = record[1].as_str().unwrap().to_owned();
    //     let rank = record[2].as_float().unwrap();
    //     let year = record[3].as_i64();
    //     links.push(Edge {
    //         source_id,
    //         target_id,
    //         rank,
    //         year,
    //     });
    // }
    links
}