use colored::Colorize;
use jornais::{newspapers, model::{JournalNew, DBInfo}};
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

    let mut load_file = false;
    let db_info_fp = std::path::Path::new("db_conn.json");

    if db_info_fp.exists() {
        let user_load = menu("There are database connection settings saved, do you want to load them?", &["Yes", "No"]);

        if user_load.as_str() == "0" {
            println!("{}", "[ Loading information to connect to database from file ]".green());
            load_file = true;
        }
    }


    let mut db_info = DBInfo {
        user: String::from(""),
        password: String::from(""),
        name: String::from(""),
        port: String::from("")
    };

    if !load_file {
        db_info.user = menu("Enter your MySQL username", &[]);
        db_info.password = menu("Enter your MySQL password", &[]);
        db_info.name = menu("Enter the name of the database you want to use", &[]);
        db_info.port = menu("Enter the port where the database server is running", &[]);

        let save_to_file = menu("Do you want to save these connection settings to a file? (This will remove previously saved settings)", &["Yes", "No"]);

        if save_to_file.as_str() == "0" {
            let mut file = match std::fs::File::create("db_conn.json") {
                Ok(f) => f,
                Err(_) => panic!("Error trying to retrieve db_conn.json file")
            };
        
            serde_json::to_writer_pretty(&mut file, &db_info).expect("Error trying to save database info");
        }
    } else {
        let data = std::fs::read_to_string(db_info_fp).expect("Error retrieving file contents of db_conn.json");

        db_info = serde_json::from_str(&data).expect("Error parsing db_conn.json");
    }
    

    println!("
        {}
        [Username] {}
        [Password] {}
        [Database] {}
        [Port]     {}    
    ",
        "[ Using the following variables to connect to the database on localhost ]".blue(), 
        db_info.user.green(), 
        db_info.password.green(), 
        db_info.name.green(), 
        db_info.port.green()
    );

    let pool = match MySqlPoolOptions::new()
    .max_connections(2)
    .connect(format!(
        "mysql://{}:{}@localhost:{}/{}",
        db_info.user,
        db_info.password,
        db_info.port,
        db_info.name
    ).as_str()).await {
        Ok(pool) => pool,
        Err(error) => panic!("
        Error connecting to database: 
        Make sure you created a database with the name 'newspapers'.
        {}
        ", error)
    };

    println!("{}", "[ Creating table to save news ]".bright_green());

    match sqlx::query(
        "CALL sys.table_exists(?, 'newspapers', @exists); SELECT @exists"
    ).bind(db_info.name).execute(&pool).await {
        Ok(exists) => println!("EXISTS {:?}", exists),
        Err(error) => println!("ERROR {}", error)
    }

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

    
}
