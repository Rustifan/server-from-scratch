use std::{net::TcpListener, io::Read};
use http::{http_request::HttpRequest};
use super::router::Router;

pub struct Server<'a>{
    socket_address: &'a str
}

impl <'a> Server<'a>{
    pub fn new(socket_address:  &'a str)->Self{
        Server {socket_address}
    }

    pub fn listen(&self){
        let tcp_listener = TcpListener::bind(self.socket_address).unwrap();
        println!("Listening on {}", self.socket_address);
        for stream in tcp_listener.incoming(){
            let mut stream = stream.unwrap();
            let mut read_buffer = [0; 1024];
            stream.read(&mut read_buffer).unwrap();

            let req: HttpRequest = String::from_utf8(read_buffer.to_vec()).unwrap().trim_matches(char::from(0)).into();
            Router::route(req, &mut stream);
        }
    }
}