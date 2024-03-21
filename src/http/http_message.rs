use std::{collections::HashMap};
use std::cmp::min;
use std::path::PathBuf;

#[allow(dead_code)]
pub struct HTTPMessage {
    pub status_code: u16,
    pub request_type: String,
    pub path: String,
    pub query: String,
    pub protocol: String,
    pub header: HashMap<String, String>,
    pub body: String,
}

#[allow(dead_code)]
impl HTTPMessage {
    pub fn new() -> Self {
        HTTPMessage {
            status_code: 200,
            request_type: String::new(),
            path: String::from("/"),
            query: String::from(""),
            protocol: String::from("HTTP/1.1"),
            header: HashMap::new(),
            body: String::new(),
        }
    }

    pub fn parse_request(data: &str) -> Result<Self, String> {
        let (info, header, body) = Self::split_request(data);

        let mut words = info.split_whitespace();
        let request_type = words.next().unwrap_or("").to_string();
        let query: String;
        let mut path = words.next().unwrap_or("/").to_string();
        if let Some(pos) = path.find('?') {
            query = path[pos..].to_string();
            path = path[..pos].to_string();
        } else {
            query = String::new();
        }
        path = Self::sanitize_path(&path);
        let protocol = words.next().unwrap_or("HTTP/1.1").to_string();

        Ok(HTTPMessage {
            status_code: if !info.is_empty() && !request_type.is_empty() { 200 } else { 500 },
            request_type,
            path,
            query,
            protocol,
            header: Self::parse_header(header),
            body: body.to_string(),
        })
    }

    // returns tuple of info, header, body as strings
    fn split_request(request: &str) -> (&str, &str, &str) {
        let mut lines = request.lines();
        let info = lines.next().unwrap_or("");
        let header_end = match info.find("\r\n\r\n") {
            Some(header_end) => header_end,
            None => return (info, &request[info.len()..], ""),
        };
        let header = &request[info.len()..header_end];
        let body = &request[header_end..];

        (info, header, body)
    }

    fn parse_header(header: &str) -> HashMap<String, String> {
        let lines = header.lines();
        let mut header_map: HashMap<String, String> = HashMap::new();

        for line in lines {
            match line.find(": ") {
                Some(split) => { header_map.insert(
                        (&line[..split]).to_string(),
                        (&line[split + 2..]).to_string()); },
                None => continue,
            }
        }

        header_map
    }

    pub fn make_response(&mut self) -> String {
        let status_text = Self::get_status_code_text(self.status_code);
        let header_text = self.get_header_as_text();

        self.add("Content-Length", &self.body.len().to_string());
        if !self.header.contains_key("Content-Type") {
            self.add("Content-Type", "text/html");
        }
        if self.protocol.is_empty() {
            self.protocol = String::from("HTTP/1.1");
        }

        String::from(
            format!("{} {} {}\n{}\r\n\r\n{}",
                &self.protocol,
                &self.status_code,
                status_text,
                header_text,
                self.body))
    }

    fn get_status_code_text<'a>(code: u16) -> &'a str {
        match code {
            200 => "OK",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "I don't know that code"
        }
    }

    fn get_header_as_text(&self) -> String {
        let mut header_text = String::new();

        for (key, value) in &self.header {
            header_text.push_str(&format!("{}: {}\n", key, value));
        }

        header_text
    }

    pub fn get(&self, field_name: &str) -> Option<&String> {
        self.header.get(field_name)
    }

    pub fn add(&mut self, field_name: &str, value: &str) {
        self.header.insert(field_name.to_string(), value.to_string());
    }

    fn sanitize_path(path: &str) -> String {
        let decoded = Self::url_decode(path);
        let mut normal_path = PathBuf::from(decoded);
        let relative_path = normal_path.strip_prefix("/").unwrap_or(&normal_path);

        relative_path.to_str().unwrap_or_default().to_string()
    }

    fn url_decode(text: &str) -> String {
        let text = text.replace('+', " ");
        let mut chars: Vec<char> = text.chars().collect();
        let mut next_percent = chars.iter().enumerate().find(|&(_, &c)| c == '%');

        while let Some((index, _)) = next_percent {
            let start = min(index + 1, chars.len());
            let end = min(index + 3, chars.len());

            let percent_byte: Vec<char> = chars.drain(index..end).collect();
            if end - start == 2 {
                if let Ok(value) = u8::from_str_radix(&percent_byte[1..3].iter().collect::<String>(), 16) {
                    if let Some(c) = std::char::from_u32(value as u32) {
                        chars.insert(index, c);
                    }
                }
            }

            next_percent = chars.iter().enumerate().find(|&(_, &c)| c == '%');
        }

        chars.iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::HTTPMessage;

    #[test]
    fn test_percent_decode() {
        assert_eq!(HTTPMessage::url_decode("%20%20%20"), "   ");
    }

    #[test]
    fn test_percent_decode_with_alphanumeric() {
        assert_eq!(HTTPMessage::url_decode("Hello%20World"), "Hello World");
    }

    #[test]
    fn test_percent_decode_with_special_characters() {
        assert_eq!(HTTPMessage::url_decode("%21%40%23%24%25%5E%26%2A%28%29"), "!@#$%^&*()");
    }

    #[test]
    fn test_percent_decode_with_mixed_characters() {
        assert_eq!(HTTPMessage::url_decode("Language%3ARust%20%26%20Web%20Development"), "Language:Rust & Web Development");
    }

    #[test]
    fn test_percent_decode_with_no_encoding() {
        assert_eq!(HTTPMessage::url_decode("NoEncodingHere"), "NoEncodingHere");
    }

    #[test]
    fn test_percent_decode_with_incomplete_percent_encoding() {
        assert_eq!(HTTPMessage::url_decode("Incomplete%2"), "Incomplete");
    }

    #[test]
    fn test_percent_decode_with_upper_and_lower_case() {
        // Percent-encoding should be case-insensitive for hex digits
        assert_eq!(HTTPMessage::url_decode("%2f%2F"), "//");
    }

    #[test]
    fn test_percent_decode_with_plus_sign_as_space() {
        // Some implementations may treat plus '+' as space ' ', others may not, this one does
        assert_eq!(HTTPMessage::url_decode("Plus+Sign"), "Plus Sign");
    }

    #[test]
    fn test_percent_decode_empty_string() {
        assert_eq!(HTTPMessage::url_decode(""), "");
    }

    #[test]
    fn test_percent_decode_with_utf8() {
        // Testing UTF-8 encoded characters (e.g., "é" as "%C3%A9")
        assert_eq!(HTTPMessage::url_decode("%C3%A9"), "é");
    }
}


