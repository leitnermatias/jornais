use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Newspaper {
    CLARIN,
    LANACION,
    ROSARIO3,
    INFOBAE,
    LACAPITAL
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct JournalNew {
    pub title: String,
    pub text: String,
    pub link: Option<String>,
    pub newspaper: Newspaper
}

#[derive(Serialize, Deserialize)]
pub struct DBInfo {
    pub user: String,
    pub password: String,
    pub name: String,
    pub port: String
}