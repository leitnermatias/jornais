use std::fmt::Display;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Newspaper {
    CLARIN,
    LANACION,
    ROSARIO3,
    INFOBAE,
    LACAPITAL
}

impl Display for Newspaper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CLARIN => write!(f, "clarin"),
            Self::LANACION => write!(f, "lanacion"),
            Self::ROSARIO3 => write!(f, "rosario3"),
            Self::INFOBAE => write!(f, "infobae"),
            Self::LACAPITAL => write!(f, "lacapital")
        }
    }
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