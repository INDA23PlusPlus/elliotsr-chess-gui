use std::net::TcpStream;

use api as backend;
use chess_network_protocol as network;
use serde::Deserialize;

pub struct Chess {
    game: backend::Game
}

impl Chess {
    pub fn new() -> Self {
        Self {
            game: backend::default_game()
        }
    }
}

// bruh
fn backend_i_to_xy(i: usize) -> (i32, i32) {
    ((i as i32 - 21) / 10, (i as i32 - 1) % 10)
}

fn backend_move_to_network_move(mv: backend::Ply) -> network::Move {
    let (start_x, start_y) = backend_i_to_xy(mv.origin);
    let (end_x, end_y) = backend_i_to_xy(mv.destination);
    network::Move {
        start_x: start_x as usize, start_y: start_y as usize,
        end_x: end_x as usize, end_y: end_y as usize,
        promotion: network::Piece::None
    }
}

fn backend_board_to_network_board(game: backend::Game) -> [[network::Piece; 8]; 8] {
    let mut new_board = [[network::Piece::None; 8]; 8];
    for y in 0..8 {
        for x in 0..8 {
            let pos = backend::Pos { rank: x as i32, file: y as i32 };
            new_board[y][x] = match game.get_tile_from_pos(pos) {
                Some(tile) => match tile {
                    backend::Tile::Pawn(backend::Color::White) => network::Piece::WhitePawn,
                    backend::Tile::Bishop(backend::Color::White) => network::Piece::WhiteBishop,
                    backend::Tile::Knight(backend::Color::White) => network::Piece::WhiteKnight,
                    backend::Tile::Rook(backend::Color::White) => network::Piece::WhiteRook,
                    backend::Tile::Queen(backend::Color::White) => network::Piece::WhiteQueen,
                    backend::Tile::King(backend::Color::White) => network::Piece::WhiteKing,
                    _ => network::Piece::None,
                },
                None => network::Piece::None,
            }
        }
    }
    new_board
}

impl Chess {

    fn try_make_move(&self, mv: network::Move) -> bool {
        let origin = backend::Pos { file: mv.start_x as i32, rank: mv.start_y as i32 };
        let destination = backend::Pos { file: mv.end_x as i32, rank: mv.end_y as i32 };
        match self.game.ply(origin, destination) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn player_to_move(&self) -> network::Color {
        match self.game.get_player() {
            backend::Color::White => network::Color::White,
            backend::Color::Black => network::Color::Black
        }
    }

    pub fn get_board(&self) -> [[network::Piece; 8]; 8] {
        backend_board_to_network_board(self.game)
    }

    pub fn try_server_move(&self, mv: network::Move) -> Option<network::ServerToClient> {
        if self.try_make_move(mv) {
            let joever = if self.game.is_checkmate() {
                match self.player_to_move() {
                    network::Color::White => network::Joever::Black,
                    network::Color::Black => network::Joever::White,
                }
            } else {
                network::Joever::Ongoing
            };
    
            Some(network::ServerToClient::State {
                board: backend_board_to_network_board(self.game),
                moves: self.get_moves(),
                joever,
                move_made: mv
            })
        } else {
            None
        }
    }

    pub fn try_client_move(&self, mv: network::Move, tcp: TcpStream) -> network::ServerToClient {
        let moved = network::ClientToServer::Move(mv);
        serde_json::to_writer(&tcp, &moved).unwrap();
        let mut de = serde_json::Deserializer::from_reader(&tcp);
        let recieve = network::ServerToClient::deserialize(&mut de).unwrap();
        recieve
    }
    
    pub fn get_moves(&self) -> Vec<network::Move> {
        self.game.get_plys().into_iter().map(|&m| backend_move_to_network_move(m)).collect()
    }
}
