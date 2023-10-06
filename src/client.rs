use serde::{Serialize, Deserialize};
use chess_network_protocol::*;

use std::io::prelude::*;
use std::net::TcpStream;

fn start() -> std::io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:5000")?;
    let mut de = serde_json::Deserializer::from_reader(&stream);

    let handshake = ClientToServerHandshake {
        server_color: Color::Black,
    };

    //send
    serde_json::to_writer(&stream, &handshake).unwrap();

    //receive
    let deserialized = ServerToClientHandshake::deserialize(&mut de)?;
    println!("Recieved: {:?}", deserialized);

    loop {

        let moved = ClientToServer::Move(Move { 
            start_x: 0, 
            start_y: 0, 
            end_x: 1, 
            end_y: 1, 
            promotion: Piece::None, 
        });
    
        // send client move
        serde_json::to_writer(&stream, &moved).unwrap();
    
        // recieve server verification
        let deserialized = ServerToClient::deserialize(&mut de)?;
        println!("Recieved: {:?}", deserialized);
    
        //receive server move
        let deserialized = ServerToClient::deserialize(&mut de)?;
        println!("Recieved: {:?}", deserialized);
    }
}