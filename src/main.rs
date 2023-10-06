mod app;
mod client;
mod server;
mod layer;

fn main() {
    let is_server: bool = true;
    
    app::run(is_server).unwrap();
}