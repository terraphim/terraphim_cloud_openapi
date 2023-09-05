use itertools::Itertools;
use terraphim_automata::load_automata;
use terraphim_automata::matcher::{find_matches, find_matches_ids, replace_matches, Dictionary};
use terraphim_pipeline::split_paragraphs;
use terraphim_pipeline::{magic_pair, magic_unpair};

fn main() {
    let paragraph = "This is the first sentence.\n\n This is the second sentence. This is the second sentence? This is the second sentence| This is the second sentence!\n\nThis is the third sentence. Mr. John Johnson Jr. was born in the U.S.A but earned his Ph.D. in Israel before joining Nike Inc. as an engineer. He also worked at craigslist.org as a business analyst.";

    for sentence in split_paragraphs(paragraph) {
        println!("Sentence {}", sentence);
    }
    let automata_url = "./data/term_to_id.json";
    let dict_hash = load_automata(automata_url).unwrap();
    let haystack =
        "I am a text with the word Organization strategic plan and bar and project calendar";
    let matches = find_matches(haystack, dict_hash.clone(), false);
    println!("Matches: {:?}", matches);
    let matches2 = replace_matches(haystack, dict_hash.clone()).unwrap();
    println!("Matches: {:?}", String::from_utf8_lossy(&matches2));
    println!("{}", &matches2.len());
    let matches3 = find_matches_ids(haystack, &dict_hash).unwrap();
    println!("Matched Ids {:?}", matches3);
    let mut v = Vec::new();
    for (a, b) in matches3.into_iter().tuple_windows() {
        v.push(magic_pair(a, b));
    }
    println!("Magic Pair {:?}", v);
    for z in v.into_iter() {
        println!("{:?}", magic_unpair(z));
    }
}
