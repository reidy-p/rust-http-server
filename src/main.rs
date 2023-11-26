mod http;

use http::{HttpMethod, HttpRequest, HttpResponse, HttpStatusCode};
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::string::String;
use std::thread::spawn;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").expect("failed to create TCP listener");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                spawn(|| {
                    handle_request(stream);
                });
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }
}

fn handle_request(mut stream: TcpStream) {
    let mut buf = [0; 1024];
    stream.read(&mut buf).expect("Failed to read stream");

    let response = match std::str::from_utf8(&buf) {
        Ok(raw_request) => {
            let request = http::HttpRequest::new(raw_request);

            if request.path.eq("/") {
                build_ok_response(None)
            } else if request.path.starts_with("/echo/") {
                let content = request.path.replace("/echo/", "");
                build_ok_response(Some(&content))
            } else if request.path.starts_with("/user-agent") {
                build_ok_response(Some(
                    &request
                        .headers
                        .get("User-Agent")
                        .expect("failed to get user-agent"),
                ))
            } else if request.path.starts_with("/files") {
                handle_file_request(request)
            } else {
                HttpResponse {
                    status_code: HttpStatusCode::NotFound,
                    content: None,
                }
            }
        }
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    flush_response(stream, response)
}

fn flush_response(mut stream: TcpStream, response: HttpResponse) {
    match stream.write(response.to_string().as_str().as_bytes()) {
        Ok(_) => {
            let _ = stream.flush();
        }
        Err(e) => panic!("failed to write response: {}", e),
    }
}

fn build_ok_response(content: Option<&str>) -> HttpResponse {
    HttpResponse {
        status_code: HttpStatusCode::Ok,
        content: content.map(|c| http::HttpContent {
            content: String::from(c),
            content_type: http::HttpContentType::TextPlain,
        }),
    }
}

fn handle_file_request(request: HttpRequest) -> HttpResponse {
    let file = request.path.replace("/files/", "");
    let args: Vec<String> = std::env::args().collect();
    let full_path = format!("{}/{}", args[2], file);
    if request.method == HttpMethod::Get {
        match std::fs::read(full_path.as_str()) {
            Ok(content) => HttpResponse {
                status_code: HttpStatusCode::Ok,
                content: Some(http::HttpContent {
                    content: String::from_utf8(content).expect("invalid content"),
                    content_type: http::HttpContentType::ApplicationOctetStream,
                }),
            },
            Err(_) => HttpResponse {
                status_code: HttpStatusCode::NotFound,
                content: None,
            },
        }
    } else {
        let mut file = File::create(full_path).unwrap();
        match request.content {
            Some(http_content) => {
                file.write_all(http_content.content.replace('\x00', "").as_bytes())
                    .expect("failed to write to file");
            }
            None => panic!("No content found"),
        }

        HttpResponse {
            status_code: HttpStatusCode::Created,
            content: None,
        }
    }
}
