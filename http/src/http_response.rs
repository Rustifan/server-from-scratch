use std::{collections::HashMap, io::{Write, Error}};

#[derive(Debug, Clone, PartialEq)]
pub struct HttpResponse<'a>{
    version: &'a str,
    status_code: &'a str,
    status_text: &'a str,
    headers: HashMap<&'a str, String>,
    body: Option<String>
}

impl <'a>Default for HttpResponse<'a> {
    fn default() -> Self {
        Self { 
            version: "HTTP/1.1", 
            status_code: "200", 
            status_text: "OK", 
            headers: HashMap::new(), 
            body: None 
        }
    }
}

impl<'a> HttpResponse<'a>{
    pub fn new(
        status_code: &'a str,
        headers: Option<HashMap<&'a str, String>>,
        body: Option<String>
    )->Self{
        let mut http_response = Self::default();
        http_response.status_code = status_code;
        match headers {
            Some(headers) => {http_response.headers = headers},
            None=>{
                http_response.headers.insert("Content-Type", "text/html".to_string());
                http_response.headers.insert("Connection", "keep-alive".to_string());
            }
        }
        http_response.status_text = match status_code {
            "200" => "OK",
            "400" => "Bad request",
            "404" => "Not Found",
            "500" => "Server error",
            _ => "Unknown"
        };
        http_response.body = body;
        http_response
    }

    pub fn get_headers_as_string(&self)->String{
        self.headers
            .clone()
            .iter()
            .fold(String::from(""), 
            |acc, (key, value)|{
            format!("{acc}{key}: {value}\r\n")
        })
    }

    fn set_content_length_header(&mut self){
        let body_length: String =  match &self.body{
            Some(body)=>body.len().to_string(),
            None=>"0".to_string()
        };
     
        self.headers.insert("Content-Length", body_length);

    }

    pub fn send_response(&mut self, stream: &mut impl Write)-> Result<(), Error>{
        self.set_content_length_header();
        stream.write_all(String::from(self.clone()).as_bytes())?;
        stream.flush()
    }
}


impl<'a> From<HttpResponse<'a>> for String {
    fn from(value: HttpResponse) -> Self {
        let HttpResponse {version, status_code, body, status_text ,..} = &value;
        let headers_string = value.get_headers_as_string();
        let body_string = match body {
            Some(body)=>body,
            None=>""
        };
        format!("{version} {status_code} {status_text}\r\n{headers_string}\r\n{body_string}")
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_headers_from_string(){
        let mut headers = HashMap::new();
        headers.insert("Content-Type", "text/html".to_string());
        headers.insert("Authentication", "Bearer 123456".to_string());

        let res = HttpResponse::new("200", Some(headers), None);       
        assert_eq!(res.get_headers_as_string(), "Content-Type: text/html\r\nAuthentication: Bearer 123456\r\n".to_string());
    }
    #[test]
    fn test_string_from_http_response(){
        let mut headers = HashMap::new();
        headers.insert("Content-Type", "text/html".to_string());
        headers.insert("Authentication", "Bearer 123456".to_string());
        let response = HttpResponse::new("404", Some(headers), Some(String::from("Hello world")));
        
        let response_string = String::from(response);
        let expected_string = "HTTP/1.1 404 Not Found\r\nContent-Type: text/html\r\nAuthentication: Bearer 123456\r\n\r\nHello world".to_string();
        assert_eq!(expected_string, response_string);
    }


}