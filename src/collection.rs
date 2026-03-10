use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::request::{Auth, HttpMethod};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedRequest {
    pub name: String,
    pub method: String,
    pub url: String,
    pub body: Option<String>,
    pub auth_type: String,
    pub username: String,
    pub password: String,
    pub token: String,
    pub headers: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Collection {
    pub requests: Vec<SavedRequest>,
}

impl SavedRequest {
    pub fn from_current(
        name: &str,
        method: Option<HttpMethod>,
        url: &str,
        body: Option<&str>,
        auth: Auth,
        username: &str,
        password: &str,
        token: &str,
        headers: &[(String, String)],
    ) -> Self {
        SavedRequest {
            name: name.to_string(),
            method: method.map(|m| m.to_string()).unwrap_or_default(),
            url: url.to_string(),
            body: body.map(|b| b.to_string()),
            auth_type: match auth {
                Auth::None => "None",
                Auth::Basic => "Basic",
                Auth::Bearer => "Bearer",
            }
            .to_string(),
            username: username.to_string(),
            password: password.to_string(),
            token: token.to_string(),
            headers: headers.to_vec(),
        }
    }

    pub fn method(&self) -> Option<HttpMethod> {
        HttpMethod::from_str(&self.method)
    }

    pub fn auth(&self) -> Auth {
        match self.auth_type.as_str() {
            "Basic" => Auth::Basic,
            "Bearer" => Auth::Bearer,
            _ => Auth::None,
        }
    }
}

fn collections_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".patchlite").join("collections")
}

fn collection_path() -> PathBuf {
    collections_dir().join("collection.json")
}

pub fn load_collection() -> Collection {
    let path = collection_path();
    if !path.exists() {
        return Collection::default();
    }
    let data = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&data).unwrap_or_default()
}

pub fn save_collection(collection: &Collection) {
    let dir = collections_dir();
    fs::create_dir_all(&dir).ok();
    let data = serde_json::to_string_pretty(collection).unwrap_or_default();
    fs::write(collection_path(), data).ok();
}
