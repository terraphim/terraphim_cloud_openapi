    //
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