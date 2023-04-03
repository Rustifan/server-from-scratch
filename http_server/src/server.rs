use std::{net::{TcpListener, TcpStream, SocketAddr}, io::{Read, ErrorKind, Write, Error}, time::{ Duration, Instant}, sync::{Arc, Mutex}, thread, collections::{LinkedList}};
use http::{http_request::{HttpRequest}};
use crate::web_socket::{handle_web_socket_upgrade, WebSocketConnections, read_web_socket_message};

use super::router::Router;

pub struct Connection{
    stream: TcpStream,
    last_time: Instant
}

impl Connection {
    pub fn get_source_address(&self)->SocketAddr{
        self.stream.peer_addr().expect("No peer address error")
    }

    pub fn write(&mut self, data: &[u8])-> Result<(), Error>{
        self.stream.write(data)?;
        self.stream.flush()
    }
}

enum ConnectionStatus{
    Close,
    Open,
    Handled,
    SocketUpgrade
}

impl Connection{
    pub fn new(stream: TcpStream)->Self{
        Connection {
            stream,
            last_time:Instant::now()
        }
    }

    pub fn is_timeout(&self, timeout: u64)->bool{
        self.last_time.elapsed() > Duration::from_secs(timeout)
    }
}

pub struct Server<'a>{
    socket_address: &'a str,
    connections: Arc<Mutex<LinkedList<Connection>>>,
    web_socket_connections: Arc<Mutex<WebSocketConnections>>
}

const MAX_THREADS: u32 = 1;

const KEEP_ALIVE_TIME: u64 = 5;

impl <'a> Server<'a>{
    pub fn new(socket_address:  &'a str)->Self{
        Server {
            socket_address, 
            connections: Arc::new(Mutex::new(LinkedList::new())),
            web_socket_connections: Arc::new(Mutex::new(WebSocketConnections::new()))
        }
    }
    

    fn set_worker_threads(&self){
        for _ in 0..MAX_THREADS{
            let connections = Arc::clone(&self.connections);
            let ws_connections = Arc::clone(&self.web_socket_connections);

            thread::spawn(move || loop {
               
                let ws_connection = ws_connections.lock().unwrap().pop();
               
                if let Some(ws_connection) = ws_connection {
                    let mut connection_ptr = ws_connection;
                    let connection = Arc::get_mut(&mut connection_ptr).expect("Failed to get mut");
                    let status = handle_web_socket_connection(&mut connection.stream);
                    match status {
                        ConnectionStatus::Open => {
                            ws_connections
                            .lock()
                            .expect("Mutec lock failed")
                            .insert(&connection_ptr);
                        },
                        _=>{}
                    }
                }


                
                let connection = connections.lock().unwrap().pop_back();
                let mut connection = match connection {
                    Some(connection)=>{
                        if connection.is_timeout(KEEP_ALIVE_TIME){
                            continue;
                        }
                        connection
                    }
                    None=>{
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                };

                let connection_status = handle_connection(&mut connection.stream);
                match connection_status {
                    ConnectionStatus::Close => continue,
                    ConnectionStatus::Handled=>{connection.last_time = Instant::now();},
                    
                    ConnectionStatus::SocketUpgrade=>{
                        ws_connections.lock().expect("to lock ws connections")
                        .insert(&Arc::new(connection));
                        continue;
                    },
                
                    _=>{}
                }
                connections.lock().unwrap().push_front(connection);
            });
        }
    }



    pub fn listen(&self){
        self.set_worker_threads();
        
        let tcp_listener = TcpListener::bind(self.socket_address).unwrap();
        println!("Listening on {}", self.socket_address);
        for stream in tcp_listener.incoming(){
            let stream = stream.unwrap();
            self.connections.lock().unwrap().push_back(Connection::new(stream));
        }
            
    }
}
            

fn handle_connection(stream: &mut TcpStream)->ConnectionStatus{
    stream.set_nonblocking(true).unwrap();
    let ip = stream.peer_addr().unwrap();
    let mut read_buffer = vec![0; 1024];
    let size = stream.read(&mut read_buffer);
    let _ = match size {
      Ok(0)=>return ConnectionStatus::Close,  
      Ok(size)=>size,  
      Err(e) if e.kind() == ErrorKind::WouldBlock => {
     
        return ConnectionStatus::Open
        }
        Err(e)=>{
            println!("{}", e);
            return ConnectionStatus::Close
        }
    };
    println!("connection - {ip}");
    let req: HttpRequest = String::from_utf8(read_buffer.to_vec()).unwrap().trim_matches(char::from(0)).into();
    
    //check if request is web socket handshake
    let ws_result = handle_web_socket_upgrade(&req, stream);
    if let Err(s) = ws_result {
        println!("{s}");
    }else{
        return ConnectionStatus::SocketUpgrade;
    }

    Router::route(req, stream);

    return ConnectionStatus::Handled
}
    

fn handle_web_socket_connection(stream: &mut TcpStream)->ConnectionStatus{
    let mut read_buffer = [0; 1024];
    let size = stream.read(&mut read_buffer);
    let size = match size {
      Ok(0)=>return ConnectionStatus::Close,  
      Ok(size)=>size,  
      Err(e) if e.kind() == ErrorKind::WouldBlock => {
     
        return ConnectionStatus::Open
        }
        Err(e)=>{
            println!("{}", e);
            return ConnectionStatus::Close
        }
    };

    let buffer = &read_buffer[..size];
    read_web_socket_message(buffer);
   

    ConnectionStatus::Open
}

