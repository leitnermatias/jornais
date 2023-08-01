use colored::Colorize;
use jornais::newspapers;

#[tokio::main]
async fn main() {

    let (
        clarin_news,
        infobae_news,
        lanacion_news,
        lacapital_news,
        rosario3_news
    ) = tokio::join!(
        newspapers::get_clarin(),
        newspapers::get_infobae(),
        newspapers::get_lanacion(),
        newspapers::get_lacapital(),
        newspapers::get_rosario3()
    );

    println!("{}", "[ Clarin news ]".green());
    clarin_news.iter().for_each(|news| {
        println!("{}\n\n", news.title.blue().bold())
    });

    println!("{}", "[ Infobae news ]".green());
    infobae_news.iter().for_each(|news| {
        println!("{}\n\n", news.title.blue().bold())
    });

    println!("{}", "[ La Nacion news ]".green());
    lanacion_news.iter().for_each(|news| {
        println!("{}\n\n", news.title.blue().bold())
    });

    println!("{}", "[ La Capital news ]".green());
    lacapital_news.iter().for_each(|news| {
        println!("{}\n\n", news.title.blue().bold())
    });

    println!("{}", "[ Rosario3 news ]".green());
    rosario3_news.iter().for_each(|news| {
        println!("{}\n\n", news.title.blue().bold())
    });

    
}
