use std::{collections::HashMap};

#[allow(dead_code)]
pub enum HTTPRequestType {
    Get,
    Post,
    Put,
    Delete,
    Update
}

#[allow(dead_code)]
pub struct HTTPMessage {
    header: HashMap<String, String>,
    pub data: String,
    pub path: String,
    pub request_type: Option<HTTPRequestType>,
}

#[allow(dead_code)]
impl HTTPMessage {
    pub fn new() -> Self {
        HTTPMessage {
            header: HashMap::new(),
            data: String::new(),
            path: String::from("/"),
            request_type: None
        }
    }

    pub fn from(data: &str) -> Self {
        println!("{data}");
        let path = data.split_whitespace().next().unwrap_or("/").to_string();
        HTTPMessage {
            header: HashMap::new(),
            data: String::new(),
            path,
            request_type: None,
        }
    }

    pub fn get(&self, field_name: &str) -> Option<&String> {
        self.header.get(field_name)
    }

    pub fn add(&mut self, field_name: &str, value: &str) {
        self.header.insert(field_name.to_string(), value.to_string());
    }
}

