use serde::Deserialize;

use std::time::Duration;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct ImportHeader {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Import {
    pub headers: Vec<ImportHeader>,
    pub refresh: Duration,
}
