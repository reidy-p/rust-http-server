use core::fmt;
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, PartialEq)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub headers: std::collections::HashMap<String, String>,
    pub content: Option<HttpContent>,
}

impl HttpRequest {
    pub fn new(raw_request: &str) -> HttpRequest {
        match raw_request.split_once("\r\n\r\n") {
            Some((headers, body)) => {
                let lines: Vec<&str> = headers.split("\r\n").collect();
                let (method, path) = Self::parse_start_line(lines[0]);
                let headers = Self::parse_headers(&lines[1..]);

                let content = Self::parse_content(body);

                HttpRequest {
                    method,
                    path,
                    headers,
                    content,
                }
            }
            None => panic!("unexpected HTTP request format"),
        }
    }

    fn parse_start_line(line: &str) -> (HttpMethod, String) {
        let res: Vec<&str> = line.split(' ').collect();

        return (
            HttpMethod::from_str(res[0]).expect("Unexpected HTTP method"),
            res[1].to_string(),
        );
    }

    fn parse_headers(headers: &[&str]) -> HashMap<String, String> {
        let mut headers_map = HashMap::new();

        for line in headers {
            line.split_once(":").and_then(|(header, value)| {
                headers_map.insert(header.to_string(), value.trim().to_string())
            });
        }

        return headers_map;
    }

    fn parse_content(content: &str) -> Option<HttpContent> {
        if content.is_empty() {
            return None;
        } else {
            return Some(HttpContent {
                content: content.to_string(),
                content_type: HttpContentType::TextPlain,
            });
        };
    }
}

#[derive(Debug, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
}

impl FromStr for HttpMethod {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            _ => Err(()),
        }
    }
}

pub enum HttpStatusCode {
    Ok = 200,
    Created = 201,
    NotFound = 404,
}

impl fmt::Display for HttpStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.canonical_reason())
    }
}

impl HttpStatusCode {
    fn canonical_reason(&self) -> &str {
        match self {
            HttpStatusCode::Ok => "200 OK",
            HttpStatusCode::Created => "201 Created",
            HttpStatusCode::NotFound => "404 Not Found",
        }
    }
}

pub struct HttpResponse {
    pub status_code: HttpStatusCode,
    pub content: Option<HttpContent>,
}

impl fmt::Display for HttpResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.content {
            Some(content) => {
                write!(
                    f,
                    "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
                    self.status_code,
                    content.content_type,
                    content.content.len(),
                    content.content
                )
            }
            None => {
                write!(f, "HTTP/1.1 {}\r\n\r\n", self.status_code)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct HttpContent {
    pub content: String,
    pub content_type: HttpContentType,
}

#[derive(Debug, PartialEq)]
pub enum HttpContentType {
    ApplicationJson,
    ApplicationOctetStream,
    TextPlain,
}

impl HttpContentType {
    fn as_str(&self) -> &'static str {
        match self {
            HttpContentType::ApplicationJson => "application/json",
            HttpContentType::ApplicationOctetStream => "application/octet-stream",
            HttpContentType::TextPlain => "text/plain",
        }
    }
}

impl fmt::Display for HttpContentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for HttpContentType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "application/json" => Ok(HttpContentType::ApplicationJson),
            "application/octet-stream" => Ok(HttpContentType::ApplicationOctetStream),
            "text/plain" => Ok(HttpContentType::TextPlain),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_get_request() {
        let raw_request = "GET /example/resource HTTP/1.1\r\n\r\n";
        let request = HttpRequest::new(raw_request);

        let expected = HttpRequest {
            method: HttpMethod::Get,
            path: String::from("/example/resource"),
            headers: HashMap::new(),
            content: None,
        };
        assert_eq!(request, expected);
    }

    #[test]
    fn test_parse_simple_get_request_with_headers() {
        let raw_request = "GET /example/resource HTTP/1.1\r\nHost: www.example.com\r\nUser-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:100.0) Gecko/20100101 Firefox/100.0\r\n\r\n";
        let request = HttpRequest::new(raw_request);

        let headers = [
            (
                String::from("Host"), 
                String::from("www.example.com")
            ),
            (
            String::from("User-Agent"),
            String::from(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:100.0) Gecko/20100101 Firefox/100.0",
            ),
        )];

        let expected = HttpRequest {
            method: HttpMethod::Get,
            path: String::from("/example/resource"),
            headers: HashMap::from(headers),
            content: None,
        };
        assert_eq!(request, expected);
    }

    #[test]
    fn test_parse_simple_post_request() {
        let raw_request = "POST /example/resource HTTP/1.1\r\nHost: www.example.com\r\nContent-Type: application/json\r\n\r\n{\"key1\": \"value1\", \"key2\": \"value2\"}";
        let request = HttpRequest::new(raw_request);

        let headers = [
            (String::from("Host"), String::from("www.example.com")),
            (
                String::from("Content-Type"),
                String::from("application/json"),
            ),
        ];

        let expected = HttpRequest {
            method: HttpMethod::Post,
            path: String::from("/example/resource"),
            headers: HashMap::from(headers),
            content: Some(HttpContent {
                content: String::from("{\"key1\": \"value1\", \"key2\": \"value2\"}"),
                content_type: HttpContentType::ApplicationJson,
            }),
        };
        assert_eq!(request, expected);
    }
}
