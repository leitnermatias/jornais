use colored::Colorize;
use jornais::{newspapers, model::JournalNew};
use tokio::{task, time};
use std::{io::{self, Write}, time::Duration};
use sqlx::{mysql::MySqlPoolOptions, Pool, MySql, Row};

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

fn _print_news(title: &str, news: Vec<JournalNew>) {
    println!("{}", title.green());
    let separator = "----------------".bright_white();

    news.iter().for_each(|news| {
        println!("\n{}\n {} \n{}", separator, news.title.blue().bold(), separator)
    });
}

async fn save_news_to_database(pool: &Pool<MySql>, news: JournalNew, newspaper_name: String) {
    let title = news.title.as_str();
    let text = news.text.as_str();
    let posted_on = "1999-12-12";
    match sqlx::query("
        SELECT COUNT(*) as count FROM news WHERE title = ?
    ").bind(title).fetch_one(pool).await {
        Ok(row) => {
            let count: i64 = row.try_get("count").expect("Couldn't get count from query");

            if count == 0 {
                match sqlx::query(
                    "INSERT INTO news (title, text, posted_on, newspaper_name) VALUES (
                        ?,
                        ?,
                        ?,
                        ?
                    )"
                )
                .bind(title)
                .bind(text)
                .bind(posted_on)
                .bind(newspaper_name)
                .execute(pool).await {
                    Ok(_) => println!("{}", "[ Saved a new title ]".green()),
                    Err(e) => println!("{}", e)
                };
            }
        },
        Err(error) => println!("{}", error)
    };


}

#[tokio::main]
async fn main() {

    let db_user = menu("Enter your MySQL username", &[]);
    let db_password = menu("Enter your MySQL password", &[]);
    let db_name = menu("Enter the name of the database you want to use", &[]);
    let db_port = menu("Enter the port where the database server is running", &[]);

    println!("
        {}
        [Username] {}
        [Password] {}
        [Database] {}
        [Port]     {}    
    ",
        "[ Using the following variables to connect to the database on localhost ]".blue(), 
        db_user.green(), 
        db_password.green(), 
        db_name.green(), 
        db_port.green()
    );

    let pool = match MySqlPoolOptions::new()
    .max_connections(2)
    .connect(format!("mysql://{db_user}:{db_password}@localhost:{db_port}/{db_name}").as_str()).await {
        Ok(pool) => pool,
        Err(error) => panic!("
        Error connecting to database: 
        Make sure you created a database with the name 'newspapers'.
        {}
        ", error)
    };

    println!("{}", "[ Creating table to save news ]".bright_green());

    match sqlx::query(
        "CREATE TABLE IF NOT EXISTS news (
            id MEDIUMINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
            title CHAR(200) NOT NULL,
            text TEXT,
            saved_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            posted_on DATE,
            newspaper_name CHAR(40) NOT NULL
        )"
    ).execute(&pool).await {
        Ok(_) => println!("{}", "[ Created table to save news ]".bright_green()),
        Err(error) => panic!("{error}")
    };

    let _ = task::spawn(async move {
        // Will execute every 20 minutes
        let mut interval = time::interval(Duration::from_secs(60 * 20));
        println!("{}", "[ Starting to look for new titles ]".bright_blue());

        loop {
            interval.tick().await;

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

            for journal_new in clarin_news {
                save_news_to_database(&pool, journal_new, String::from("clarin")).await
            }

            for journal_new in infobae_news {
                save_news_to_database(&pool, journal_new, String::from("infobae")).await
            }

            for journal_new in lanacion_news {
                save_news_to_database(&pool, journal_new, String::from("la nacion")).await
            }

            for journal_new in lacapital_news {
                save_news_to_database(&pool, journal_new, String::from("la capital")).await
            }

            for journal_new in rosario3_news {
                save_news_to_database(&pool, journal_new, String::from("rosario3")).await
            }
        }
    }).await;


    // loop {
    //     let user_input = menu("Please select which newspapers to get latest news",
    //         &[
    //         "All",
    //         "Clarin",
    //         "Infobae",
    //         "La Nacion",
    //         "La Capital",
    //         "Rosario3",
    //         "Exit"
    //     ]);

        
    
    //     match user_input.as_str() {
    //         "0" => {
    //             let (
    //                 clarin_news,
    //                 infobae_news,
    //                 lanacion_news,
    //                 lacapital_news,
    //                 rosario3_news
    //             ) = tokio::join!(
    //                 newspapers::get_clarin(),
    //                 newspapers::get_infobae(),
    //                 newspapers::get_lanacion(),
    //                 newspapers::get_lacapital(),
    //                 newspapers::get_rosario3()
    //             );
    
    //             print_news("Clarin", clarin_news);
    //             print_news("Infobae", infobae_news);
    //             print_news("La Nacion", lanacion_news);
    //             print_news("La Capital", lacapital_news);
    //             print_news("Rosario3", rosario3_news);
    //         }
    
    //         "1" => {
    //             let clarin_news = newspapers::get_clarin().await;
    
    //             print_news("Clarin", clarin_news);
    //         }
    //         "2" => {
    //             let infobae_news = newspapers::get_infobae().await;
    
    //             print_news("Infobae", infobae_news);
    //         }
    //         "3" => {
    //             let lanacion_news = newspapers::get_lanacion().await;
    
    //             print_news("La Nacion", lanacion_news);
    //         }
    //         "4" => {
    //             let lacapital_news = newspapers::get_lacapital().await;
    
    //             print_news("La Capital", lacapital_news);
    //         }
    //         "5" => {
    //             let rosario3_news = newspapers::get_rosario3().await;
    
    //             print_news("Rosario3", rosario3_news);
    //         }
    //         "6" => {
    //             break
    //         }
    //         _ => {
    //             println!("Invalid option")
    //         }
    //     }

    // }    

    
}
