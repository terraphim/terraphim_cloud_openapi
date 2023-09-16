use ahash::{AHashMap, HashMap};
use memoize::memoize;
use regex::Regex;
use std::collections::hash_map::Entry;
use std::collections::BTreeMap;
use std::mem;
use terraphim_automata::load_automata;
use terraphim_automata::matcher::{find_matches_ids, Dictionary};
use unicode_segmentation::UnicodeSegmentation;
#[derive(Debug, Clone)]
pub struct RoleGraph {
    // role filter
    role: String,
    nodes: AHashMap<u64, Node>,
    edges: AHashMap<u64, Edge>,
    automata_url: String,
    dict_hash: AHashMap<String, Dictionary>,
}
impl RoleGraph {
    pub fn new(role: String, automata_url: &str) -> Self {
        let dict_hash = load_automata(automata_url).unwrap();
        Self {
            role,
            nodes: AHashMap::new(),
            edges: AHashMap::new(),
            automata_url: automata_url.to_string(),
            dict_hash: dict_hash,
        }
    }
    pub fn query(&self, query_string: &str) {
        println!("performing query");
        let nodes = find_matches_ids(query_string, &self.dict_hash).unwrap_or(Vec::new());
        for node_id in nodes.iter() {
            println!("Matched node {:?}", node_id);
            let node = self.nodes.get(node_id).unwrap();
            println!("Node Rank {}", node.rank);
            println!("Node connected to Edges {:?}", node.connected_with);
            for each_edge_key in node.connected_with.iter() {
                let each_edge = self.edges.get(each_edge_key).unwrap();
                println!("Edge {:?}", each_edge);
            }
            // sort nodes by rank
            //sort edges by rank
            // sort article id by rank
            // node rank is a weight for edge and edge rank is a weight for article_id
            // create hashmap of output with article_id, rank to dedupe articles in output
            // normalise output rank from 1 to number of records
            // pre-sort article_id by rank using BtreeMap
        }
    }
    pub fn add_or_update_article(&mut self, article_id: String, x: u64, y: u64) {
        let edge = magic_pair(x, y);
        let edge = self.init_or_update_edge(edge, article_id);
        self.init_or_update_node(x, &edge);
        self.init_or_update_node(y, &edge);
    }
    fn init_or_update_node(&mut self, node_id: u64, edge: &Edge) {
        match self.nodes.entry(node_id) {
            Entry::Vacant(_) => {
                let node = Node::new(node_id, edge.clone());
                self.nodes.insert(node.id, node);
            }
            Entry::Occupied(entry) => {
                let mut node = entry.into_mut();
                node.rank += 1;
                node.connected_with.push(edge.id);
            }
        };
    }
    fn init_or_update_edge(&mut self, edge_key: u64, article_id: String) -> Edge {
        let edge = match self.edges.entry(edge_key) {
            Entry::Vacant(_) => {
                let edge = Edge::new(edge_key, article_id);
                self.edges.insert(edge.id, edge.clone());
                edge
            }
            Entry::Occupied(entry) => {
                let mut edge = entry.into_mut();
                *edge.doc_hash.entry(article_id).or_insert(1) += 1;
                let edge_read = edge.clone();
                edge_read
            }
        };
        edge
    }
}
#[derive(Debug, Clone)]
pub struct Edge {
    // id of the node
    id: u64,
    rank: u64,
    // hashmap document_id, rank
    doc_hash: AHashMap<String, u32>,
}
impl Edge {
    pub fn new(id: u64, article_id: String) -> Self {
        let mut doc_hash = AHashMap::new();
        doc_hash.insert(article_id, 1);
        Self {
            id,
            rank: 1,
            doc_hash: doc_hash,
        }
    }
}
// Node represent single concept
#[derive(Debug, Clone)]
pub struct Node {
    id: u64,
    // number of co-occureneces
    rank: u64,
    connected_with: Vec<u64>,
}
impl Node {
    fn new(id: u64, edge: Edge) -> Self {
        let mut connected_with = Vec::new();
        connected_with.push(edge.id);
        Self {
            id,
            rank: 1,
            connected_with: connected_with,
        }
    }
    // pub fn sort_edges_by_value(&self) {
    //     // let count_b: BTreeMap<&u64, &Edge> =
    //     // self.connected_with.iter().map(|(k, v)| (v, k)).collect();
    //     // for (k, v) in self.connected_with.iter().map(|(k, v)| (v.rank, k)) {
    //     // println!("k {:?} v {:?}", k, v);
    //     // }
    //     println!("Connected with {:?}", self.connected_with);
    // }
}

#[macro_use]
extern crate lazy_static;
lazy_static! {
    static ref RE: Regex = Regex::new(r"[?!|]\s+").unwrap();
}
pub fn split_paragraphs(paragraphs: &str) -> Vec<&str> {
    let sentences = UnicodeSegmentation::split_sentence_bounds(paragraphs);
    let parts = sentences.flat_map(|sentence| RE.split(sentence.trim()));
    parts.map(|part| part.trim()).collect()
}

/// Combining two numbers into a unique one: pairing functions.
/// It uses "elegant pairing" (https://odino.org/combining-two-numbers-into-a-unique-one-pairing-functions/).
/// also using memoize macro with Ahash hasher
#[memoize(CustomHasher: ahash::AHashMap)]
pub fn magic_pair(x: u64, y: u64) -> u64 {
    if x >= y {
        x * x + x + y
    } else {
        y * y + x
    }
}

// Magic unpair
// func unpair(z int) (int, int) {
//   q := int(math.Floor(math.Sqrt(float64(z))))
//     l := z - q * q

//   if l < q {
//       return l, q
//   }

//   return q, l - q
// }
#[memoize(CustomHasher: ahash::AHashMap)]
pub fn magic_unpair(z: u64) -> (u64, u64) {
    let q = (z as f64).sqrt().floor() as u64;
    let l = z - q * q;
    if l < q {
        return (l, q);
    } else {
        return (q, l - q);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let paragraph = "This is the first sentence.\n\n This is the second sentence. This is the second sentence? This is the second sentence| This is the second sentence!\n\nThis is the third sentence. Mr. John Johnson Jr. was born in the U.S.A but earned his Ph.D. in Israel before joining Nike Inc. as an engineer. He also worked at craigslist.org as a business analyst.";
        for sentence in split_paragraphs(paragraph) {
            println!("{}", sentence);
        }
        // assert_eq!(result, 4);
    }
}
