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
pub struct Document {
    id: String,
    // matched to edges
    matched_to: Vec<u64>,
}


#[derive(Debug, Clone)]
pub struct RoleGraph {
    // role filter
    role: String,
    nodes: AHashMap<u64, Node>,
    edges: AHashMap<u64, Edge>,
    documents: AHashMap<String, Document>,
    automata_url: String,
    dict_hash: AHashMap<String, Dictionary>,
    // counters to make ranking queries easier 
    node_count: u64,
    edge_count: u64,
    document_count: u64,
    // normalization of weights for weighted average
    normalizer: f64,
    weight_nodes: f64,
    weight_edges: f64,
    weight_articles: f64,
}
impl RoleGraph {
    pub fn new(role: String, automata_url: &str) -> Self {
        let dict_hash = load_automata(automata_url).unwrap();
        Self {
            role,
            nodes: AHashMap::new(),
            edges: AHashMap::new(),
            documents: AHashMap::new(),
            automata_url: automata_url.to_string(),
            dict_hash: dict_hash,
            node_count: 0,
            edge_count: 0,
            document_count: 0,
            normalizer: 1.0,
            weight_nodes: 1.0,
            weight_edges: 1.0,
            weight_articles: 1.0,
        }
    }
    //  Query the graph using a query string, returns a list of article ids ranked and weighted by weighted mean average of node rank, edge rank and article rank
    // node rank is a weight for edge and edge rank is a weight for article_id
    // create hashmap of output with article_id, rank to dedupe articles in output
    // normalise output rank from 1 to number of records
    // pre-sort article_id by rank using BtreeMap
    //  overall weighted average is calculated as (node_rank*edge_rank*article_rank)/(node_rank+edge_rank+article_rank)
    pub fn query(&self, query_string: &str) {
        println!("performing query");
        // FIXME: handle case when no matches found with empty non empty vector - otherwise all ranks will blow up
        let nodes = find_matches_ids(query_string, &self.dict_hash).unwrap_or(Vec::from([1]));
        let mut non_sorted_vector=Vec::new();
        // let mut sorted_vector_by_rank_weighted: Vec<_>=Vec::new();
        for node_id in nodes.iter() {
            println!("Matched node {:?}", node_id);
            let node = self.nodes.get(node_id).unwrap();
            let node_rank=node.rank;
            println!("Node Rank {}", node_rank);
            println!("Node connected to Edges {:?}", node.connected_with);
            for each_edge_key in node.connected_with.iter() {
                let each_edge = self.edges.get(each_edge_key).unwrap();
                println!("Edge Details{:?}", each_edge);
                let edge_rank=each_edge.rank;
                for (article_id, rank) in each_edge.doc_hash.iter() {
                    // final rank is a weighted average of node rank and edge rank and article rank
                    //  weighted average  can be calculated: sum of (weight*rank)/sum of weights for each node, edge and article.
                    //  rank is a number of co-occurences, the output will be normalised within query results output, potentially it can be normalised across all articles
                    // see cleora train function
                    // TODO: design question: duplicated articles in output should be removed, what shall happens with rank?
                    println!("Article id {} Rank {}", article_id, rank);
                    non_sorted_vector.push((article_id, rank));

                }
            }
            
            //TODO: create top_k_nodes function where
            // sort nodes by rank
            // TODO create top_k_edges function where
            //sort edges by rank
            // TODO create top_k_articles function where
            // sort article id by rank

        }
            println!("Vector to be Sorted{:?}", non_sorted_vector);
            // sorted_vector.sort_by(|a, b| b.1.cmp(&a.1));
            non_sorted_vector.sort_by(|a, b| b.1.cmp(a.1));
            println!("Sorted Vector by rank {:?}", non_sorted_vector);
            let node_len=self.nodes.len() as u64;
            println!("Node Length {}", node_len);
            let edge_len=self.edges.len() as u64;
            println!("Edge Length {}", edge_len);
            let article_len=non_sorted_vector.len() as u64;
            println!("Article Length {}", article_len);
            let normalizer=f64::from_bits(node_len+edge_len+article_len);
            let weight_node=f64::from_bits(node_len)/normalizer;
            let weight_edge=f64::from_bits(edge_len)/normalizer;
            let weight_article=f64::from_bits(article_len)/normalizer;
            println!("Weight Node {}", weight_node);
            println!("Weight Edge {}", weight_edge);
            println!("Weight Article {}", weight_article);
            // for (article_id,rank) in non_sorted_vector.iter(){
            //     let weighted_rank=(weight_node*node_rank as f64)+(weight_edge*edge_rank as f64)+(weight_article*rank as f64)/(weight_node+weight_edge+weight_article);
            //     println!("Article id {} Weighted Rank {}", article_id, weighted_rank);
            //     sorted_vector_by_rank_weighted.push((article_id, weighted_rank));
            // }
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
                self.node_count += 1;
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
                self.edge_count += 1;
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
    doc_hash: AHashMap<String, u64>,
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
