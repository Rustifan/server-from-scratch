use server::Server;

mod server;
mod router;
mod handler;

fn main() {
    let server = Server::new("127.0.0.1:8080");
    server.listen();
 }
 