use terraphim_automata::{Dictionary, Matched, load_automata, find_matches};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

use redis::{FromRedisValue, Value};

use redis_derive::{FromRedisValue, ToRedisArgs};

use crate::settings::{Settings, self};

#[derive(Debug, Deserialize, Serialize)]
pub struct Edge {
    source: String,
    target: String,
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
pub fn mock_get_edges() -> Vec<Edge> {
    let edges = vec![
        Edge {
            source: "node1".to_string(),
            target: "node2".to_string(),
            rank: 0.5,
            year: Some(2021),
        },
        Edge {
            source: "node2".to_string(),
            target: "node3".to_string(),
            rank: 0.8,
            year: Some(2022),
        },
    ];
    edges
}

pub fn get_edges(
    settings: &Settings,
    nodes: &[String],
    years: Option<&[&str]>,
    limits: i64,
    mnodes: &std::collections::HashSet<String>,
) -> Vec<Edge> {
    let mut links = Vec::new();
    // let mut nodes_set = std::collections::HashSet::new();
    let mut years_set:Vec<String> = Vec::new();
    let url = settings.redis_url.clone();
    let client = redis::Client::open(url).unwrap();
    let mut conn = client.get_connection().unwrap();
    let graph_name = "cord19medical";
    let query = if let Some(years) = years {
        let years: Vec<String> = years.iter().map(|year| year.to_string()).collect();
        format!(
            "WITH $ids as ids
            MATCH (e:entity)-[r]->(t:entity)
            WHERE e.id IN ids AND r.year IN $years
            RETURN DISTINCT e.id, t.id, max(r.rank), r.year
            ORDER BY r.rank DESC LIMIT {}",
            limits
        )
    } else {
        format!(
            "WITH $ids as ids
            MATCH (e:entity)-[r]->(t:entity)
            WHERE e.id IN ids
            RETURN DISTINCT e.id, t.id, max(r.rank), r.year
            ORDER BY r.rank DESC LIMIT {}",
            limits
        )
    };
    let params = if let Some(years) = years {
        redis::cmd("GRAPH.QUERY")
            .arg(graph_name)
            .arg(query)
            .arg("ids")
            .arg(nodes)
            .arg("years")
            .arg(years)
            .arg("limits")
            .arg(limits)
            .query::<redis::Value>(&mut conn)
            .unwrap()
    } else {
        redis::cmd("GRAPH.QUERY")
            .arg(graph_name)
            .arg(query)
            .arg("ids")
            .arg(nodes)
            .arg("limits")
            .arg(limits)
            .query::<redis::Value>(&mut conn)
            .unwrap()
    };
    let result_set = params;
    println!("Result set: {:?}", result_set);
    // for record in result_set {
    //     let source = record[0].as_str().unwrap().to_owned();
    //     let target = record[1].as_str().unwrap().to_owned();
    //     let rank = record[2].as_float().unwrap();
    //     let year = record[3].as_i64();
    //     if !mnodes.contains(&source) {
    //         nodes_set.insert(source.clone());
    //     } else {
    //         println!("Node {} excluded", source);
    //     }
    //     if !mnodes.contains(&target) {
    //         nodes_set.insert(target.clone());
    //     } else {
    //         println!("Node {} excluded", target);
    //     }
    //     if let Some(year) = year {
    //         years_set.insert(year);
    //     }
    //     links.push(Edge {
    //         source,
    //         target,
    //         rank,
    //         year,
    //     });
    // }
    links
}