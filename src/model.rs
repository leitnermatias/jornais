use serde::{Serialize, Deserialize};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct JournalNew {
    pub title: String,
    pub text: String,
    pub link: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct DBInfo {
    pub user: String,
    pub password: String,
    pub name: String,
    pub port: String
}