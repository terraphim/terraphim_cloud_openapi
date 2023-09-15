use itertools::Itertools;
use terraphim_automata::load_automata;
use terraphim_automata::matcher::{find_matches, find_matches_ids, replace_matches, Dictionary};
use terraphim_pipeline::split_paragraphs;
use terraphim_pipeline::{magic_pair, magic_unpair, RoleGraph};
use ulid::Ulid;
fn main() {
    let paragraph = "This is the first sentence.\n\n This is the second sentence. This is the second sentence? This is the second sentence| This is the second sentence!\n\nThis is the third sentence. Mr. John Johnson Jr. was born in the U.S.A but earned his Ph.D. in Israel before joining Nike Inc. as an engineer. He also worked at craigslist.org as a business analyst.";
    println!("Sentence segmentation test");
    for sentence in split_paragraphs(paragraph) {
        println!("Sentence {}", sentence);
    }
    println!("System operator role");
    let role = "system operator".to_string();
    let automata_url = "https://system-operator.s3.eu-west-2.amazonaws.com/term_to_id.json";
    let dict_hash = load_automata(automata_url).unwrap();
    let query = "I am a text with the word Life cycle concepts and bar and Trained operators and maintainers, project direction, some bingo words Paradigm Map and project planning, then again: some bingo words Paradigm Map and project planning, then repeats: Trained operators and maintainers, project direction";
    let matches = find_matches(query, dict_hash.clone(), false);
    println!("Matches: {:?}", matches);
    let matches2 = replace_matches(query, dict_hash.clone()).unwrap();
    println!("Matches: {:?}", String::from_utf8_lossy(&matches2));
    println!("{}", &matches2.len());
    let matches3 = find_matches_ids(query, &dict_hash).unwrap();
    println!("Matched Ids {:?}", matches3);
    let mut v = Vec::new();
    let mut rolegraph = RoleGraph::new(role, automata_url.to_string());
    let article_id = Ulid::new().to_string();
    for (a, b) in matches3.into_iter().tuple_windows() {
        println!("a {} b {}", a, b);
        rolegraph.add_or_update_article(article_id.clone(), a, b);
        v.push(magic_pair(a, b));
    }
    println!("Magic Pairs {:?}", v);

    println!("Magic unpar");
    for z in v.into_iter() {
        println!("{:?}", magic_unpair(z));
    }
    println!("{:?}", rolegraph);
    println!("Query graph");
    // TODO: now query edges given nodes, shall query will be cantor product?
    // query: matched query to nodes
    // for each node return list of all edges (in this node)
    // sort by node rank, edge_rank, article_id rank
    rolegraph.query();
}
