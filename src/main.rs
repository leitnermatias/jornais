use colored::Colorize;
use jornais::{newspapers, model::JournalNew};
use std::io::{self, Write};

fn menu(title: &str, options: &[&str]) -> String {
    println!("{}", title.bright_green());
    
    for (index, option) in options.iter().enumerate() {
        let option_number = format!("[{index}]").green();
        println!("{option_number} {option}")
    }

    let mut user_input = String::new();
    let stdin = io::stdin();

    print!("{}", "> ".bright_green().blink());
    io::stdout().flush().expect("Failed to flush output");
    let input_result = stdin.read_line(&mut user_input);

    match input_result {
        Ok(_) => {},
        Err(error) => println!("{}", error)
    }

    String::from(user_input.trim())

}

fn print_news(title: &str, news: Vec<JournalNew>) {
    println!("{}", title.green());
    let separator = "----------------".bright_white();

    news.iter().for_each(|news| {
        println!("\n{}\n {} \n{}", separator, news.title.blue().bold(), separator)
    });
}

#[tokio::main]
async fn main() {

    loop {
        let user_input = menu("Please select which newspapers to get latest news",
            &[
            "All",
            "Clarin",
            "Infobae",
            "La Nacion",
            "La Capital",
            "Rosario3",
            "Exit"
        ]);
    
        match user_input.as_str() {
            "0" => {
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
    
                print_news("Clarin", clarin_news);
                print_news("Infobae", infobae_news);
                print_news("La Nacion", lanacion_news);
                print_news("La Capital", lacapital_news);
                print_news("Rosario3", rosario3_news);
            }
    
            "1" => {
                let clarin_news = newspapers::get_clarin().await;
    
                print_news("Clarin", clarin_news);
            }
            "2" => {
                let infobae_news = newspapers::get_infobae().await;
    
                print_news("Infobae", infobae_news);
            }
            "3" => {
                let lanacion_news = newspapers::get_lanacion().await;
    
                print_news("La Nacion", lanacion_news);
            }
            "4" => {
                let lacapital_news = newspapers::get_lacapital().await;
    
                print_news("La Capital", lacapital_news);
            }
            "5" => {
                let rosario3_news = newspapers::get_rosario3().await;
    
                print_news("Rosario3", rosario3_news);
            }
            "6" => {
                break
            }
            _ => {
                println!("Invalid option")
            }
        }

    }    

    
}
