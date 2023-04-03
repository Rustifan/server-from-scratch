use std::{net::TcpStream, collections::{HashMap, LinkedList}, sync::{Mutex, Arc}, io::Write, slice::Iter};
use sha1::{Sha1, Digest};
use http::{http_request::HttpRequest, http_response::HttpResponse};
use base64::{Engine as _, engine::general_purpose};

use crate::server::Connection;

pub struct WebSocketConnections{
    connections_map: HashMap<String, Arc<Connection>>,
    connections_list: LinkedList<Arc<Connection>>
}

impl WebSocketConnections{
    pub fn new()->Self{
        WebSocketConnections { connections_map: HashMap::new(), connections_list: LinkedList::new() }
    }

    pub fn insert(&mut self, connection: &Arc<Connection>)->String{
        let connection_string = hash_to_string(connection.get_source_address().to_string());
        self.connections_map.insert(connection_string.clone(), Arc::clone(connection));
        self.connections_list.push_front(Arc::clone(connection));

        connection_string
    }

    pub fn pop(&mut self)->Option<Arc<Connection>>{
        let connection = self.connections_list.pop_back()?;
        let connection_string = hash_to_string(connection.get_source_address().to_string());
        self.connections_map.remove(&connection_string);
        
        Some(connection)
    }

    pub fn send_to_all(&mut self, buf: &[u8]){
        for connection in self.connections_list.iter_mut(){
        
            let connection = Arc::get_mut(connection);
            let connection = match connection {
                None=>continue,
                Some(connection)=>connection
            };
            let _ = connection.write(buf);
        }
    }

}


fn validate_header(key: &str, match_value: &str, headers: &HashMap<String, String>)->bool{
    match headers.get(key) {
        None=>false,
        Some(value)=>value.as_str() == match_value
    }
}

fn validate_upgrade_headers(headers: &HashMap<String, String>)->bool{
    
    let validate_pairs = [
            ("Upgrade", "websocket"),
            ("Connection", "Upgrade"),
            ("Sec-WebSocket-Version", "13")
        ];
        
    for (key, value) in validate_pairs.iter(){
        if !validate_header(key, value, headers){
            return false
        }
    }
        
    true
}

pub fn handle_web_socket_upgrade(req: &HttpRequest, stream: &mut TcpStream)->Result<(), &'static str>{
    let headers = &req.headers;
    if !validate_upgrade_headers(&headers) {
        return Err("Not ws upgrade");
    }
    let sec_web_socket_key = match headers.get("Sec-WebSocket-Key") {
        Some(key) => key,
        None=>return Err("No sec key in headers")
    };

    let mut response_headers = HashMap::new();
    response_headers.insert("Upgrade", "websocket".to_string());
    response_headers.insert("Connection", "Upgrade".to_string());
    response_headers.insert("Sec-WebSocket-Accept", get_web_socket_accept_key(sec_web_socket_key));

    let mut response =  HttpResponse::new("101", Some(response_headers), None);
    let result = response.send_response(stream);    
    if let Ok(_) = result {
        return Ok(())
    }

    return Err("Something went wrong while writing to TCP stream")
}
    

pub fn get_web_socket_accept_key(request_key: &String)->String{
    let ws_uid = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let magic_string = format!("{request_key}{ws_uid}");
    let mut hasher = Sha1::new();
    hasher.update(magic_string);
    let hash_result = hasher.finalize();
    
    general_purpose::STANDARD.encode(hash_result)
}

pub fn hash_to_string(str: String)->String{
    let mut hasher = Sha1::new();
    hasher.update(str);
    let res = hasher.finalize();
    
    format!("{:x}", res)
  
}

#[derive(Debug)]
enum Opcode {
    ContinuationFrame,
    TextFrame,
    BinaryFrame,
    ConnectionCloseFrame,
    PingFrame,
    PongFrame,
    Unknown
}
impl From<u8> for Opcode{
    fn from(value: u8) -> Self {
        match value {
           0 => Opcode::ContinuationFrame,
           1 => Opcode::TextFrame,
           2 => Opcode::BinaryFrame,
           8 => Opcode::ConnectionCloseFrame,
           9 => Opcode::PingFrame,
           10 => Opcode::PongFrame,
           _ => Opcode::Unknown
        }
    }
}


pub fn read_web_socket_message(buffer: &[u8])->Option<()>{
    let mut bytes = buffer.iter();
    let first_byte = bytes.next()?;
    let fin = first_byte >> 7;
    let opcode: Opcode = (first_byte & 0xF).into();
    
    let second_byte = bytes.next()?;
    let mask_bit = second_byte >> 7;
    let payload_len = get_payload_len(second_byte);
    
    let masking_key_bytes: Vec<&u8> = bytes
    .clone()
    .take(4)
    .collect();
    bytes.nth(3).unwrap();

    let masked_payload = bytes.take(payload_len);
    let unmasked_payload = masked_payload.enumerate().map(|(i, byte)|{
        let mask_index = i % 4;
        let mask = masking_key_bytes[mask_index];
        return byte ^ mask;
    }).collect::<Vec<_>>();

    let payload = std::str::from_utf8(&unmasked_payload).unwrap();
    println!("fin - {fin}");
    println!("opcode - {opcode:?}");
    println!("mask_bit - {mask_bit}");
    println!("payload_len - {payload_len}");
    println!("payload - {payload}");


    Some(())
}

pub fn get_payload_len(second_byte: &u8)->usize{
    let first_len_option = second_byte & 0b01111111;
    if first_len_option < 126 {
        return usize::from(first_len_option);
    }

    panic!("Not implemented")
}
    

    
