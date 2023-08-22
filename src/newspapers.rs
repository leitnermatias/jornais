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
                let a_tags = get_elements("a.link-new", &inner_dom, inner_parser);

                let h2 = h2s.first().expect("h2 tag should exist inside article tag");
                let h3 = h3s.first().expect("h3 tag should exist inside article tag");
                let a = a_tags.first().expect("a tag should exist inside article tag");

                if latest_news.iter().any(|journal_new| journal_new.title == h2.inner_text(inner_parser)) {
                    return
                }
                
                let link = a.as_tag().expect("Failed to convert to HTML tag").attributes().get("href").expect("Failed to get href for a tag").unwrap().as_utf8_str();


                latest_news.push(JournalNew { title: String::from(h2.inner_text(inner_parser)), text: String::from(h3.inner_text(inner_parser)), link: Some(String::from(link))});

            });


        },
        Err(error) => println!("{}", error)
    }

    latest_news
}

pub async fn get_infobae() -> Vec<JournalNew> {
    let first_page_load = reqwest::get("https://www.infobae.com/ultimas-noticias-america/").await;
    let mut latest_news: Vec<JournalNew> = vec![];

    match first_page_load {
        Ok(response) => {
            let response_html = response.text().await.unwrap();

            let dom = tl::parse(&response_html, tl::ParserOptions::default()).unwrap();
            let parser = dom.parser();

            let a_tags = get_elements("a.feed-list-card", &dom, parser);

            a_tags.iter().for_each(|node| {
                let a_tag_html = node.inner_html(parser);

                let inner_dom = tl::parse(&a_tag_html, tl::ParserOptions::default()).unwrap();
                let inner_parser = inner_dom.parser();

                let h2 = get_elements("h2.feed-list-card-headline-lean", &inner_dom, inner_parser);
                let div = get_elements("div.deck", &inner_dom, inner_parser);

                let title = h2.first().expect("h2 should exist inside a tag").inner_text(inner_parser);

                if latest_news.iter().any(|journal_new| journal_new.title == title) {
                    return
                }

                let mut journal_new = JournalNew {
                    title: String::from(title),
                    text: String::from(""),
                    link: None
                };

                if div.first().is_some() {
                    let text = div.first().expect("div should exist inside a tag").inner_text(inner_parser);
                    journal_new.text = String::from(text);
                }

                latest_news.push(journal_new)
            })
        },
        Err(error) => println!("{}", error)
    } 

    latest_news
}

pub async fn get_lanacion() -> Vec<JournalNew> {
    let first_page_load = reqwest::get("https://www.lanacion.com.ar/ultimas-noticias/").await;
    let mut latest_news: Vec<JournalNew> = vec![];

    match first_page_load {
        Ok(response) => {
            let response_html = response.text().await.unwrap();

            let dom = tl::parse(&response_html, tl::ParserOptions::default()).unwrap();
            let parser = dom.parser();

            let article_tags = get_elements("article.mod-article", &dom, parser);

            article_tags.iter().for_each(|node| {
                let article_tag_html = node.inner_html(parser);

                let inner_dom = tl::parse(&article_tag_html, tl::ParserOptions::default()).unwrap();
                let inner_parser = inner_dom.parser();

                let a_tags = get_elements("a.com-link", &inner_dom, inner_parser);

                let title = a_tags.first().expect("a tag should exist inside article").inner_text(inner_parser);

                if latest_news.iter().any(|journal_new| journal_new.title == title) {
                    return
                }

                latest_news.push(JournalNew { title: String::from(title), text: String::from(""), link: None });
            })
        },
        Err(error) => println!("{}", error)
    }

    latest_news
}

pub async fn get_lacapital() -> Vec<JournalNew> {
    let first_page_load = reqwest::get("https://www.lacapital.com.ar/secciones/ultimo-momento.html").await;
    let mut latest_news: Vec<JournalNew> = vec![];

    match first_page_load {
        Ok(response) => {
            let response_html = response.text().await.unwrap();

            let dom = tl::parse(&response_html, tl::ParserOptions::default()).unwrap();
            let parser = dom.parser();

            let h2_tags = get_elements("h2.entry-title", &dom, parser);

            h2_tags.iter().for_each(|node| {
                let title = node.inner_text(parser);

                if latest_news.iter().any(|journal_new| journal_new.title == title) {
                    return
                }

                latest_news.push(JournalNew { title: String::from(title), text: String::from(""), link: None })
            })
        },
        Err(error) => println!("{}", error)
    }

    latest_news
}

pub async fn get_rosario3() -> Vec<JournalNew> {
    let first_page_load = reqwest::get("https://www.rosario3.com/seccion/ultimas-noticias/").await;
    let mut latest_news: Vec<JournalNew> = vec![];

    match first_page_load {
        Ok(response) => {
            let response_html = response.text().await.unwrap();

            let dom = tl::parse(&response_html, tl::ParserOptions::default()).unwrap();
            let parser = dom.parser();

            let h2_tags = get_elements("h2.title", &dom, parser);

            h2_tags.iter().for_each(|node| {
                let title = node.inner_text(parser);

                if latest_news.iter().any(|journal_new| journal_new.title == title) {
                    return
                }

                latest_news.push(JournalNew { title: String::from(title), text: String::from(""), link: None })
            })
        },
        Err(error) => println!("{}", error)
    }

    latest_news
}