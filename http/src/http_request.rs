use std::{collections::HashMap, str::Lines};

#[derive(Debug, PartialEq, Clone)]
pub enum Resource {
    Path(String)
}


#[derive(Debug, PartialEq, Clone)]
pub enum Method{
    Get,
    Post,
    Uninitialized,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Version{
    V1_1,
    V2_0,
    Uninitialized
}

#[derive(Debug, Clone)]
pub struct HttpRequest{
    pub method: Method,
    pub version: Version,
    pub resource: Resource,
    pub headers: HashMap<String, String>,
    pub body: String
}

impl From<&str> for Method {
    fn from(s: &str) -> Method{
        match s {
            "GET" => Method::Get,
            "POST" => Method::Post,
            _ => Method::Uninitialized
        }
    }
}

impl From<&str> for Version {
    fn from(value: &str) -> Version {
        match value {
            "HTTP/1.1" => Version::V1_1,
            _ => Version::Uninitialized
        }
    }
}

impl From<&str> for HttpRequest {
    fn from(data: &str) -> Self {
        let data_str = data.to_string();
        let mut lines = data_str.lines();
        let request_line = lines.next();
        let (method, version, resource) = parse_request_line(request_line);
        let headers = parse_headers(&mut lines);
        let body = lines.collect::<String>();

        HttpRequest { 
            method, 
            version, 
            resource, 
            headers, 
            body
        }

        
    }
}

impl From<String> for HttpRequest {
    fn from(value: String) -> Self {
       value[0..].into()
    }
}

fn parse_headers(lines: &mut Lines)->HashMap<String, String>{
    let mut headers: HashMap<String, String> = HashMap::new();
    while let Some(line) = lines.next() {
        if line.is_empty(){
            break;
        }
        let delimiter = ":";
        if !line.contains(delimiter) {
            continue;
        }
        let mut split = line.split(delimiter);
        let key = split.next().unwrap_or("").trim();
        let value = split.next().unwrap_or("").trim();
        if !key.is_empty() {
            headers.insert(key.to_string(), value.to_string());
        }
    }

    headers
}

fn parse_request_line(line: Option<&str>)->(Method, Version, Resource){
    match line {
        Some(str)=>{
            let mut split_by_spaces = str.split_whitespace();
            let method: Method = split_by_spaces
            .next()
            .unwrap_or("")
            .into();

            let resource: Resource = Resource::Path(
                split_by_spaces
                .next()
                .unwrap_or("")
                .to_string()
            );
            
            let version: Version = split_by_spaces
            .next()
            .unwrap_or("")
            .into();
            
           

            (method, version, resource)
        },
        None => (
            Method::Uninitialized, 
            Version::Uninitialized, 
            Resource::Path("".to_string())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_into(){
        let method: Method = "GET".into();
        assert_eq!(method, Method::Get);
    }   
    #[test]
    fn test_version_into(){
        let version: Version = "HTTP/1.1".into();
        assert_eq!(version, Version::V1_1);
    }
    #[test]
    fn test_read_http() {
        let test_string: String = String::from("GET /greeting HTTP/1.1\r\nHost: localhost:3000\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\nHello world"); 
        let mut headers_expected = HashMap::new(); 
        headers_expected.insert("Host".into(), "localhost".into());
        headers_expected.insert("Accept".into(), "*/*".into());
        headers_expected.insert("User-Agent".into(), "curl/7.64.1".into());

        let req: HttpRequest = test_string.into(); 
        assert_eq!(Method::Get, req.method); 
        assert_eq!(Version::V1_1, req.version); 
        assert_eq!(Resource::Path("/greeting".to_string()), req.resource); 
        assert_eq!(headers_expected, req.headers); 
        assert_eq!("Hello world", req.body);
    }

}

