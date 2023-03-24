use std::{net::{TcpListener, TcpStream}, io::{Read, ErrorKind}, time::{ Duration, Instant}, sync::{Arc, Mutex}, thread, collections::LinkedList};
use http::{http_request::{HttpRequest}};
use super::router::Router;


pub struct Connection{
    stream: TcpStream,
    last_time: Instant
}

impl Connection{
    pub fn new(stream: TcpStream)->Self{
        Connection {
            stream,
            last_time: Instant::now(),
        }
    }

    pub fn is_timeout(&self, timeout: u64)->bool{
        let now = Instant::now();
        let lasted = now.duration_since(self.last_time).as_secs();
        //TODO
        lasted > timeout
    }
}

pub struct Server<'a>{
    socket_address: &'a str,
    connections: Arc<Mutex<LinkedList<Connection>>>
}

const MAX_THREADS: u32 = 1;

const KEEP_ALIVE_TIME: u64 = 5;

impl <'a> Server<'a>{
    pub fn new(socket_address:  &'a str)->Self{
        Server {
            socket_address, 
            connections: Arc::new(Mutex::new(LinkedList::new()))
        }
    }
    

    fn set_worker_threads(&self){
        for _ in 0..MAX_THREADS{
            let connections = Arc::clone(&self.connections);
            thread::spawn(move || loop {
                let connection = connections.lock().unwrap().pop_back();
                
                let mut connection = match connection {
                    Some(connection)=>{
                        if connection.is_timeout(KEEP_ALIVE_TIME){
                            println!("Ban");
                            continue;
                        }
                        connection
                    }
                    None=>{
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                };

                let bring_back_connection = handle_connection(&mut connection.stream);
                if bring_back_connection {
                    connection.last_time = Instant::now();
                    connections.lock().unwrap().push_front(connection)
                }
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
            

fn handle_connection(stream: &mut TcpStream)->bool{
    stream.set_nonblocking(true).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let ip = stream.peer_addr().unwrap();
    let mut read_buffer = vec![0; 1024];
    let size = stream.read(&mut read_buffer);
    let _ = match size {
      Ok(0)=>return false,  
      Ok(size)=>size,  
      Err(e) if e.kind() == ErrorKind::WouldBlock => {
     
        return true
        }
        Err(e)=>{
            println!("{}", e);
            return false
        }
    };
    println!("connection - {ip}");
    let req: HttpRequest = String::from_utf8(read_buffer.to_vec()).unwrap().trim_matches(char::from(0)).into();
    Router::route(req, stream);
    return true
}
    