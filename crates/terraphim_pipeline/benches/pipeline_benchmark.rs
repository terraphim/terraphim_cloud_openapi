use criterion::{criterion_group, criterion_main, Criterion};
use itertools::Itertools;
use terraphim_automata::load_automata;
use terraphim_automata::matcher::{find_matches, find_matches_ids, replace_matches, Dictionary};
use terraphim_pipeline::split_paragraphs;
use terraphim_pipeline::{magic_pair, magic_unpair, RoleGraph};
use ulid::Ulid;
use lazy_static::lazy_static;
use ahash::{AHashMap, HashMap};

lazy_static! {
    static ref AUTOMATA: AHashMap<String, Dictionary> = {
        let dict_hash = load_automata("https://system-operator.s3.eu-west-2.amazonaws.com/term_to_id.json").unwrap();
        dict_hash
    };
}

fn bench_find_matches_ids(c: &mut Criterion) {
    let query = "I am a text with the word Life cycle concepts and bar and Trained operators and maintainers, project direction, some bingo words Paradigm Map and project planning, then again: some bingo words Paradigm Map and project planning, then repeats: Trained operators and maintainers, project direction";
    
    c.bench_function_over_inputs(
        "find_matches_ids",
        move |b, &&size| {
            let query = query.repeat(size);
            b.iter(|| find_matches_ids(&query, &AUTOMATA).unwrap())
        },
        &[1, 10, 100, 1000],
    );
}

fn bench_find_matches(c: &mut Criterion) {
    let query = "I am a text with the word Life cycle concepts and bar and Trained operators and maintainers, project direction, some bingo words Paradigm Map and project planning, then again: some bingo words Paradigm Map and project planning, then repeats: Trained operators and maintainers, project direction";
    
    c.bench_function("find_matches", |b| {
        b.iter(|| find_matches(query, AUTOMATA.clone(), false))
    });
}
fn bench_split_paragraphs(c: &mut Criterion) {
    let paragraph = "This is the first sentence.\n\n This is the second sentence. This is the second sentence? This is the second sentence| This is the second sentence!\n\nThis is the third sentence. Mr. John Johnson Jr. was born in the U.S.A but earned his Ph.D. in Israel before joining Nike Inc. as an engineer. He also worked at craigslist.org as a business analyst.";
    c.bench_function("split_paragraphs", |b| {
        b.iter(|| split_paragraphs(paragraph))
    });
}
fn bench_replace_matches(c: &mut Criterion) {
    let query = "I am a text with the word Life cycle concepts and bar and Trained operators and maintainers, project direction, some bingo words Paradigm Map and project planning, then again: some bingo words Paradigm Map and project planning, then repeats: Trained operators and maintainers, project direction";
    
    c.bench_function("replace_matches", |b| {
        b.iter(|| replace_matches(query, AUTOMATA.clone()).unwrap())
    });
}
criterion_group!(benches, bench_find_matches_ids,bench_find_matches,bench_split_paragraphs,bench_replace_matches);
criterion_main!(benches);