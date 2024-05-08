use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconCollection {
    pub prefix: String,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub suffixes: Option<HashMap<String, String>>,
    pub last_modified: usize,
    pub info: Collection,
    pub icons: HashMap<String, HashMap<String, Value>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Collection {
    pub name: String,
    pub total: usize,
    pub author: Author,
    pub license: License,
    pub samples: Vec<String>,
    pub height: Option<usize>,
    pub category: Option<String>,
    pub palette: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct License {
    pub title: String,
    pub spdx: String,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub prefix: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub icons: Vec<String>,
    pub total: usize,
    pub limit: usize,
    pub start: usize,
    pub collections: HashMap<String, Collection>,
    pub request: SearchRequest,
}
