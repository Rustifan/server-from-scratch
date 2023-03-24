use std::{collections::HashMap, env, fs};

use http::{
    http_request::{HttpRequest, Resource},
    http_response::HttpResponse,
};

pub trait Handler {
    fn handle(req: &HttpRequest) -> HttpResponse;
    fn load_file(file_name: &str) -> Option<String> {
        let default_path = format!("{}/public", env!("CARGO_MANIFEST_DIR"));
        let public_path = env::var("PUBLIC_PATH").unwrap_or(default_path);
        let full_path = format!("{public_path}/{file_name}");

        fs::read_to_string(full_path).ok()
    }
}

pub struct StaticPageHandler;
pub struct PageNotFoundHandler;

impl Handler for PageNotFoundHandler {
    fn handle(_: &HttpRequest) -> HttpResponse {
        HttpResponse::new("404", None, Self::load_file("NotFound.html"))
    }
}

impl Handler for StaticPageHandler {
    fn handle(req: &HttpRequest) -> HttpResponse {
        match &req.resource {
            Resource::Path(s) => {
                let route: Vec<&str> = s.split("/").collect();
                match route[1] {
                    "" => HttpResponse::new("200", None, Self::load_file("index.html")),
                    path => match Self::load_file(path) {
                        Some(contents) => {
                            let headers = get_headers_base_on_extension(path);
                            HttpResponse::new("200", Some(headers), Some(contents))
                        }
                        None => PageNotFoundHandler::handle(req)
                    },
                }
            }
        }
    }
}

fn get_headers_base_on_extension(file_name: &str) -> HashMap<&str, String> {
    let mut headers: HashMap<&str, String> = HashMap::new();
    let key = "Content-Type";
    match file_name.split('.').last() {
        Some("css") => {
            headers.insert(key, "text/css".to_string());
        }
        Some("js") => {
            headers.insert(key, "text/javascript".to_string());
        }
        Some("html") => {
            headers.insert(key, "text/html".to_string());
        }
        _ => {
            headers.insert(key, "text/plain".to_string());
        }
    }

    headers
}
