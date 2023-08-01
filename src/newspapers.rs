use tl::{VDom, Node, Parser};

use crate::model::JournalNew;

fn get_elements<'a>(selector: &str, dom: &VDom<'a>, parser: &Parser<'a>) -> Vec<Node<'a>> {
    let selected = dom
    .query_selector(selector)
    .expect("The query selector should be valid and exist");

    let mut result: Vec<Node> = vec![];

    selected.for_each(|element| {
        let node = element.get(parser).expect("The element should be gettable from the parser");

        result.push(node.clone());
    });

    result
}

pub async fn get_clarin() -> Vec<JournalNew> {
    let first_page_load = reqwest::get("https://www.clarin.com/ultimo-momento/").await;
    let mut latest_news: Vec<JournalNew> = vec![];

    match first_page_load {
        Ok(response) => {
            let response_html = response.text().await.unwrap();

            let dom = tl::parse(&response_html, tl::ParserOptions::default()).unwrap();
            let parser = dom.parser();
            let articles = dom.query_selector("article")
            .expect("article tag should exist inside clarin page");

            articles.for_each(|article| {
                let node_elem = article.get(parser).expect("parser should be able to get article tag");

                let inner_html = node_elem.inner_html(parser);

                let inner_dom = tl::parse(&inner_html,tl::ParserOptions::default()).unwrap();
                
                let inner_parser = inner_dom.parser();

                let h2s = get_elements("h2", &inner_dom, inner_parser);
                let h3s = get_elements("h3.summary", &inner_dom, inner_parser);

                let h2 = h2s.first().expect("h2 tag should exist inside article tag");
                let h3 = h3s.first().expect("h3 tag should exist inside article tag");

                latest_news.push(JournalNew { title: String::from(h2.inner_text(inner_parser)), text: String::from(h3.inner_text(inner_parser)) });

            });


        },
        Err(error) => println!("{}", error)
    }
    latest_news
}