use std::{collections::HashMap};

#[allow(dead_code)]
pub struct HTTPMessage {
    header: HashMap<String, String>,
    pub data: String,
    pub path: String,
    pub request_type: String,
}

#[allow(dead_code)]
impl HTTPMessage {
    pub fn new() -> Self {
        HTTPMessage {
            header: HashMap::new(),
            data: String::new(),
            path: String::from("/"),
            request_type: String::new(),
        }
    }

    pub fn parse_request(data: &str) -> Result<Self, String> {
        let mut words = data.split_whitespace();
        let request_type = words.next().unwrap_or("").to_string();
        let path = words.next().unwrap_or("/").to_string();

        Ok(HTTPMessage {
            header: HashMap::new(),
            data: String::new(),
            path,
            request_type,
        })
    }

    pub fn make_response(&self) -> String {
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<h1>literally mowserver</h1>".to_string()
    }

    pub fn get(&self, field_name: &str) -> Option<&String> {
        self.header.get(field_name)
    }

    pub fn add(&mut self, field_name: &str, value: &str) {
        self.header.insert(field_name.to_string(), value.to_string());
    }
}

