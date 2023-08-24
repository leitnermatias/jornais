use tl::{VDom, Node, Parser, HTMLTag};

use crate::model::{JournalNew, Newspaper};

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

fn query_node<'a>(selector: &str, node_tag: &HTMLTag, parser: &Parser<'a>) -> Node<'a> {
    let select = node_tag.query_selector(parser, selector).expect("h2 should exist inside article").next().unwrap();

    select.get(parser).unwrap().clone()

}

fn get_attribute(attribute: &str, node: &Node) -> String {
    let node_tag = node.as_tag().unwrap();
    String::from(node_tag.attributes().get(attribute).expect("Failed to get attribute for a tag").unwrap().as_utf8_str())
}

pub async fn get_clarin() -> Vec<JournalNew> {
    let first_page_load = reqwest::get("https://www.clarin.com/ultimo-momento/").await;
    let mut latest_news: Vec<JournalNew> = vec![];

    match first_page_load {
        Ok(response) => {
            let response_html = response.text().await.unwrap();

            let dom = tl::parse(&response_html, tl::ParserOptions::default()).unwrap();
            let parser = dom.parser();
            let articles = get_elements("article", &dom, parser);

            articles.iter().for_each(|node| {
                let node_tag = node.as_tag().unwrap();

                let h2 = query_node("h2", node_tag, parser);
                let h3 = query_node("h3.summary", node_tag, parser);
                let a = query_node("a.link-new", node_tag, parser);
                let link = get_attribute("href", &a);

                if latest_news.iter().any(|journal_new| journal_new.link == Some(String::from(link.clone()))) {
                    return
                }
                
                latest_news.push(
                    JournalNew { 
                        title: String::from(h2.inner_text(parser)), 
                        text: String::from(h3.inner_text(parser)), 
                        link: Some(String::from(link)),
                        newspaper: Newspaper::CLARIN
                    });

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
                let node_tag = node.as_tag().unwrap();

                let title = query_node("h2.feed-list-card-headline-lean", node_tag, parser);
                let text = query_node("div.deck", node_tag, parser);

                let link = get_attribute("href", node);

                if latest_news.iter().any(|journal_new| journal_new.link == Some(format!("http://infobae.com{}", link.clone()))) {
                    return
                }

                let journal_new = JournalNew {
                    title: String::from(title.inner_text(parser)),
                    text: String::from(text.inner_text(parser)),
                    link: Some(format!("http://infobae.com{}", link)),
                    newspaper: Newspaper::INFOBAE
                };

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
                let node_tag = node.as_tag().unwrap();


                let a_tag = query_node("a.com-link", node_tag, parser);

                let title = a_tag.inner_text(parser);

                let link = get_attribute("href", &a_tag);

                if latest_news.iter().any(|journal_new| journal_new.link == Some(format!("http://lanacion.com{}", link.clone()))) {
                    return
                }

                latest_news.push(
                    JournalNew { 
                        title: String::from(title), 
                        text: String::from(""), 
                        link: Some(format!("http://lanacion.com{}", link)),
                        newspaper: Newspaper::LANACION
                    });
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

            let articles = get_elements("article.ultimas-noticias-entry-container", &dom, parser);

            articles.iter().for_each(|node| {
                let node_tag = node.as_tag().expect("unable to convert node to tag");

                let h2_node = query_node("h2.entry-title", node_tag, parser);
                let a_node = query_node("a.cover-link", node_tag, parser);

                let title = h2_node.inner_text(parser);
                let link = get_attribute("href", &a_node);

                if latest_news.iter().any(|journal_new| journal_new.link == Some(link.clone())) {
                    return
                }

                latest_news.push(
                    JournalNew { 
                        title: String::from(title), 
                        text: String::from(""), 
                        link: Some(link),
                        newspaper: Newspaper::LACAPITAL
                    })
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

            let a_tags = get_elements("a.cover-link", &dom, parser);

            a_tags.iter().for_each(|node| {
                let link = get_attribute("href", node);
                let title = get_attribute("title", node);


                if latest_news.iter().any(|journal_new| journal_new.link == Some(format!("http://rosario3.com{}", String::from(link.clone())))) {
                    return
                }

                latest_news.push(
                    JournalNew { 
                        title: String::from(title), 
                        text: String::from(""), 
                        link: Some(format!("http://rosario3.com{}", String::from(link))),
                        newspaper: Newspaper::ROSARIO3 
                    })
            })
        },
        Err(error) => println!("{}", error)
    }

    latest_news
}