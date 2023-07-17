use terraphim_pipeline::split_paragraphs;

fn main() {
    let paragraph = "This is the first sentence.\n\n This is the second sentence. This is the second sentence? This is the second sentence| This is the second sentence!\n\nThis is the third sentence. Mr. John Johnson Jr. was born in the U.S.A but earned his Ph.D. in Israel before joining Nike Inc. as an engineer. He also worked at craigslist.org as a business analyst.";
        
    for sentence in split_paragraphs(paragraph) {
        println!("Sentence {}", sentence);
    }
    
}