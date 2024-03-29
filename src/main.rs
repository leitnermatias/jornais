use colored::Colorize;
use jornais::{newspapers, model::{JournalNew, DBInfo, Newspaper}};
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

fn format_news_to_html(title: String, journal_news: Vec<JournalNew>, newspaper: Newspaper) -> String {

    let mut news_html = String::from(format!(r#"
        <h1 class="{}-title">{title}</h1>
    "#, newspaper));

    for news in journal_news {
        let formatted = format!(r#"
        <div class="news {}">
            <button class="hideButton">v</button>
            <a target="_blank" href="{}">
                <h3>{}</h3>
            </a>
            <b>{}</b>
        </div>
        "#,
        news.newspaper,
        news.link.unwrap_or(String::from("")),
        news.title,
        news.text,
        );

        news_html += formatted.as_str();
    }

    news_html

}

#[tokio::main]
async fn main() {

    let args: Vec<String> = std::env::args().collect();

    if args[1] != "nosave" {

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
    } else {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char); // Clear the screen
        let _ = task::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60 * 5));
            loop {
                interval.tick().await;
    
                let (
                    clarin_news,
                    rosario3_news,
                    infobae_news,
                    lacapital_news,
                    lanacion_news
                ) = tokio::join!(
                    newspapers::get_clarin(),
                    newspapers::get_rosario3(),
                    newspapers::get_infobae(),
                    newspapers::get_lacapital(),
                    newspapers::get_lanacion()
                );

                
                
                let clarin_html = format_news_to_html(String::from("Clarin"), clarin_news, Newspaper::CLARIN);
                let rosario3_html = format_news_to_html(String::from("Rosario3"), rosario3_news, Newspaper::ROSARIO3);
                let infobae_html = format_news_to_html(String::from("Infobae"), infobae_news, Newspaper::INFOBAE);
                let lacapital_html = format_news_to_html(String::from("La Capital"), lacapital_news, Newspaper::LACAPITAL);
                let lanacion_html = format_news_to_html(String::from("La Nacion"), lanacion_news, Newspaper::LANACION);

                let styles = r#"
                <style>
                    body {
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        background-color: rgba(44, 74, 83, 0.74);
                        color: rgb(0, 0, 0);
                        text-align: center;
                    }

                    .news {
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        width: 75%;
                        background-color: rgba(196, 188, 177, 0.288);
                        padding: 10px;
                        border-radius: 5px;
                        border: 2px solid rgba(247, 203, 121, 0.363);
                        margin-bottom: 10px;
                    }

                    .news > b {
                        font-size: 12px;
                        text-align: justify;
                    }
                    
                    h3 {
                        font-size: 24px;
                    }

                    h1 {
                        color: rgb(0, 0, 0);
                    }

                    .hideButton {
                        align-self: flex-start;
                        font-size: 16px;
                        background-color: rgba(255, 191, 52, 0.699);
                        border: none;
                        border-radius: 5px;
                        padding: 5px 10px
                    }

                    a {
                        text-decoration: none;
                        color: rgb(2, 51, 92);
                    }

                    a:hover {
                        text-decoration: underline;
                    }

                    #counters {
                        display: flex;
                        gap: 15px;
                        font-size: 15px;
                        background-color: rgba(196, 188, 177, 0.288);
                        border-radius: 5px;
                        padding: 10px
                    }

                    #toolbar {
                        position: sticky;
                        top: 0px;
                        right: 0px;
                        background-color: rgba(196, 188, 177, 0.288);
                        margin-bottom: 10px;
                        width: 100%;
                        border-radius: 5px;
                        display: flex;
                        align-items: center;
                        justify-content: space-evenly;
                        padding: 10px 0px;
                    }
                    
                </style>
                "#;

                let script = r#"
                <script defer>
                    const $ = (selector, searchIn) => searchIn ? searchIn.querySelectorAll(selector) : document.querySelectorAll(selector)
                    const $1 = (selector, searchIn) => searchIn ? searchIn.querySelector(selector) : document.querySelector(selector)
                    
                    window.onload = function() {
                        const hideButtons = $(".hideButton")

                        hideButtons.forEach(button => {
                            button.hiding = false
                            
                            button.addEventListener('click', () => {
                                const parent = button.parentNode

                                button.hiding = !button.hiding
                                button.innerText = button.hiding ? '>' : "v"

                                const title = $1("h3", parent)
                                const text = $1("b", parent)

                                title.style.display = button.hiding ? 'none' : 'block'
                                text.style.display = button.hiding ? 'none' : 'block'
                            })
                        })

                        const rosario3Counter = $1(".rosario3Counter")
                        const clarinCounter = $1(".clarinCounter")
                        const infobaeCounter = $1(".infobaeCounter")
                        const lanacionCounter = $1(".lanacionCounter")
                        const lacapitalCounter = $1(".lacapitalCounter")

                        let counters = {
                            rosario3: 0,
                            clarin: 0,
                            infobae: 0,
                            lanacion: 0,
                            lacapital: 0
                        }

                        $(".news").forEach(news => {
                            const newspaper = news.className.split(" ")[1]
                            counters[newspaper] += 1
                        })

                        rosario3Counter.innerText = counters.rosario3.toString()
                        clarinCounter.innerText = counters.clarin.toString()
                        infobaeCounter.innerText = counters.infobae.toString()
                        lanacionCounter.innerText = counters.lanacion.toString()
                        lacapitalCounter.innerText = counters.lacapital.toString()

                        const rosario3Checkbox = $1(".rosario3-checkbox")
                        const clarinCheckbox = $1(".clarin-checkbox")
                        const infobaeCheckbox = $1(".infobae-checkbox")
                        const lanacionCheckbox = $1(".lanacion-checkbox")
                        const lacapitalCheckbox = $1(".lacapital-checkbox")

                        rosario3Checkbox.addEventListener('click', () => {

                            $(".news.rosario3").forEach(news => {
                                news.style.display = rosario3Checkbox.checked ? 'block' : 'none'
                            })

                            $1(".rosario3-title").style.display = rosario3Checkbox.checked ? 'block' : 'none'
                        })

                        clarinCheckbox.addEventListener('click', () => {

                            $(".news.clarin").forEach(news => {
                                news.style.display = clarinCheckbox.checked ? 'block' : 'none'
                            })

                            $1(".clarin-title").style.display = clarinCheckbox.checked ? 'block' : 'none'
                        })
                        
                        infobaeCheckbox.addEventListener('click', () => {

                            $(".news.infobae").forEach(news => {
                                news.style.display = infobaeCheckbox.checked ? 'block' : 'none'
                            })

                            $1(".infobae-title").style.display = infobaeCheckbox.checked ? 'block' : 'none'
                        })

                        lanacionCheckbox.addEventListener('click', () => {

                            $(".news.lanacion").forEach(news => {
                                news.style.display = lanacionCheckbox.checked ? 'block' : 'none'
                            })

                            $1(".lanacion-title").style.display = lanacionCheckbox.checked ? 'block' : 'none'
                        })

                        lacapitalCheckbox.addEventListener('click', () => {

                            $(".news.lacapital").forEach(news => {
                                news.style.display = lacapitalCheckbox.checked ? 'block' : 'none'
                            })

                            $1(".lacapital-title").style.display = lacapitalCheckbox.checked ? 'block' : 'none'
                        })

                    }


                </script>
                "#;
                
                let html_template = format!(r#"
                <!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>Noticias</title>
                    {styles}
                    {script}
                </head>
                <body>
                    <div id="toolbar">
                        <div>
                            <input checked class="rosario3-checkbox" type="checkbox">
                            <span>Rosario3</span>
                        </div>
                        <div>
                            <input checked class="clarin-checkbox" type="checkbox">
                            <span>Clarin</span>
                        </div>
                        <div>
                            <input checked class="infobae-checkbox" type="checkbox">
                            <span>Infobae</span>
                        </div>
                        <div>
                            <input checked class="lanacion-checkbox" type="checkbox">
                            <span>La Nacion</span>
                        </div>
                        <div>
                            <input checked class="lacapital-checkbox" type="checkbox">
                            <span>La Capital</span>
                        </div>
                    </div>
                    <div id="counters">
                        <span>Rosario3: <span class="rosario3Counter">0</span> noticias</span>-
                        <span>Clarin: <span class="clarinCounter">0</span> noticias</span>-
                        <span>Infobae: <span class="infobaeCounter">0</span> noticias</span>-
                        <span>La Nacion: <span class="lanacionCounter">0</span> noticias</span>-
                        <span>La Capital: <span class="lacapitalCounter">0</span> noticias</span>
                    </div>
                    {rosario3_html}
                    {clarin_html}
                    {infobae_html}
                    {lanacion_html}
                    {lacapital_html}
                </body>
                </html>
                "#);


                let now = chrono::offset::Local::now();

                let timestamp = format!("[ {} ]", now.format("%d-%m-%Y %H:%M"));
                std::fs::write("jornais.html", html_template).expect("Error writing HTML file");
                println!("{} {}\n", timestamp.on_green().black(), "[ HTML FILE UPDATED ]".on_bright_green().black());
            }
        }).await;
    }


    
}
