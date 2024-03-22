use std::{collections::HashMap};
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
    pub bin_data: Vec<u8>,
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
            bin_data: Vec::new(),
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
            bin_data: Vec::new(),
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

    pub fn make_response(&mut self) -> Vec<u8> {
        if !self.header.contains_key("Content-Type") {
            let content_type = Self::get_content_type_by_path(&self.path).to_string();
            self.add("Content-Type", &content_type);
        }
        if self.protocol.is_empty() {
            self.protocol = String::from("HTTP/1.1");
        }
        self.add("Content-Length", &(self.body.len() + self.bin_data.len()).to_string());

        let status_text = Self::get_status_code_text(self.status_code);
        let header_text = self.get_header_as_text();

        let mut response = format!("{} {} {}\n{}\r\n\r\n{}",
            &self.protocol,
            &self.status_code,
            status_text,
            header_text,
            self.body).as_bytes().to_vec();
        response.extend_from_slice(&self.bin_data);

        response
    }

    fn get_content_type_by_path(path: &str) -> &str {
        let extension = path.split(".").collect::<Vec<&str>>().into_iter().next_back();

        match extension.unwrap_or("") {
            "html" => "text/html",
            "css" => "text/css",
            "png" => "image/png",
            "jpg" => "image/jpg",
            "ttf" => "font/ttf",
            "odf" => "font/odf",
            "ico" => "image/ico",
            "js" => "script/js",
            _ => "text/plain"
        }
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
        let normal_path = PathBuf::from(decoded);
        let relative_path = normal_path.strip_prefix("/").unwrap_or(&normal_path);

        relative_path.to_str().unwrap_or_default().to_string()
    }

    fn url_decode(text: &str) -> String {
        let mut decoded_bytes: Vec<u8>= Vec::new();
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '%' {
                let mut hex = String::new();
                if let Some(c1) = chars.next() {
                    if c1 != '%' {
                        if let Some(c2) = chars.next() {
                            hex.push(c1);
                            hex.push(c2);
                            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                                decoded_bytes.push(byte);
                            }
                        }
                    } else {
                        decoded_bytes.push(b'%');
                    }
                }

            } else if c == '+' {
                decoded_bytes.push(b' ');
            } else {
                decoded_bytes.push(c as u8);
            }
        }

        String::from_utf8(decoded_bytes).unwrap_or_default()
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

    #[test]
    fn test_percent_percent_escape() {
        assert_eq!(HTTPMessage::url_decode("%%%20%%"), "% %");
    }
}


