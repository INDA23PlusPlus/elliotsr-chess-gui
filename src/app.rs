use ggez::conf::{WindowSetup, WindowMode};
use ggez::glam::{vec2, Vec2};
use ggez::{Context, ContextBuilder, GameResult};
use ggez::graphics::{self, Color, Image, Mesh, FillOptions, Rect, DrawParam, Text};
use ggez::event::{self, EventHandler};

use chess_network_protocol as network;
use crate::layer;

pub fn run(is_server: bool) -> GameResult {

    let (mut ctx, event_loop) = ContextBuilder::new("chess", "elliotsr")
        .window_setup(WindowSetup::default().title("chess - by elliotsr"))
        .window_mode(WindowMode::default().dimensions(BOARD_SIZE_PX as f32, BOARD_SIZE_PX as f32))
        .add_resource_path("res")
        .build()?;

    let game = Game::new(&mut ctx, is_server)?;
    
    event::run(ctx, event_loop, game)
}

const BACKGROUND_COLOR: Color = Color::new(0.1, 0.1, 0.1, 1.0);
const TILE_LIGHT_COLOR: Color = Color::new(1.0, 0.95, 0.9, 1.0);
const TILE_DARK_COLOR: Color = Color::new(0.2, 0.2, 0.4, 1.0);
const TILE_HOVER_COLOR: Color = Color::new(1.0, 1.0, 0.6, 0.8);
const TILE_SELECT_COLOR: Color = Color::new(1.0, 0.9, 0.6, 1.0);
const MOVE_PREVIEW_COLOR: Color = Color::new(0.3, 1.0, 0.3, 0.8);
const PLAYER_TURN_HINT_COLOR: Color = Color::new(0.3, 1.0, 0.3, 1.0);

const TURN_HINT_RADIUS: f32 = 8.0;

const TILE_SIZE_PX: f32 = 64.0;
const BOARD_SIZE_PX: f32 = 8.0 * TILE_SIZE_PX;

const UV_WIDTH: f32 = 1.0 / 6.0;
const UV_HEIGHT: f32 = 1.0 / 2.0;

const WHITE_KING_UV: Rect = Rect::new(0.0 * UV_WIDTH, 0.0 * UV_HEIGHT, UV_WIDTH, UV_HEIGHT);
const WHITE_QUEEN_UV: Rect = Rect::new(1.0 * UV_WIDTH, 0.0 * UV_HEIGHT, UV_WIDTH, UV_HEIGHT);
const WHITE_BISHOP_UV: Rect = Rect::new(2.0 * UV_WIDTH, 0.0 * UV_HEIGHT, UV_WIDTH, UV_HEIGHT);
const WHITE_KNIGHT_UV: Rect = Rect::new(3.0 * UV_WIDTH, 0.0 * UV_HEIGHT, UV_WIDTH, UV_HEIGHT);
const WHITE_ROOK_UV: Rect = Rect::new(4.0 * UV_WIDTH, 0.0 * UV_HEIGHT, UV_WIDTH, UV_HEIGHT);
const WHITE_PAWN_UV: Rect = Rect::new(5.0 * UV_WIDTH, 0.0 * UV_HEIGHT, UV_WIDTH, UV_HEIGHT);

const BLACK_KING_UV: Rect = Rect::new(0.0 * UV_WIDTH, 1.0 * UV_HEIGHT, UV_WIDTH, UV_HEIGHT);
const BLACK_QUEEN_UV: Rect = Rect::new(1.0 * UV_WIDTH, 1.0 * UV_HEIGHT, UV_WIDTH, UV_HEIGHT);
const BLACK_BISHOP_UV: Rect = Rect::new(2.0 * UV_WIDTH, 1.0 * UV_HEIGHT, UV_WIDTH, UV_HEIGHT);
const BLACK_KNIGHT_UV: Rect = Rect::new(3.0 * UV_WIDTH, 1.0 * UV_HEIGHT, UV_WIDTH, UV_HEIGHT);
const BLACK_ROOK_UV: Rect = Rect::new(4.0 * UV_WIDTH, 1.0 * UV_HEIGHT, UV_WIDTH, UV_HEIGHT);
const BLACK_PAWN_UV: Rect = Rect::new(5.0 * UV_WIDTH, 1.0 * UV_HEIGHT, UV_WIDTH, UV_HEIGHT);

fn tile_to_screen(col: usize, row: usize) -> (f32, f32) {
    let x = col as f32 * TILE_SIZE_PX;
    let y = (7 - row) as f32 * TILE_SIZE_PX;
    (x, y)
}

fn screen_to_tile((x, y): (f32, f32)) -> (usize, usize) {
    let col = (x / TILE_SIZE_PX) as usize;
    let row = 7 - (y / TILE_SIZE_PX) as usize;
    (col, row)
}

fn get_piece_uv(piece: network::Piece) -> Rect {
    match piece {
        network::Piece::BlackPawn => BLACK_PAWN_UV,
        network::Piece::BlackKnight => BLACK_KNIGHT_UV,
        network::Piece::BlackBishop => BLACK_BISHOP_UV,
        network::Piece::BlackRook => BLACK_ROOK_UV,
        network::Piece::BlackQueen => BLACK_QUEEN_UV,
        network::Piece::BlackKing => BLACK_KING_UV,
        network::Piece::WhitePawn => WHITE_PAWN_UV,
        network::Piece::WhiteKnight => WHITE_KNIGHT_UV,
        network::Piece::WhiteBishop => WHITE_BISHOP_UV,
        network::Piece::WhiteRook => WHITE_ROOK_UV,
        network::Piece::WhiteQueen => WHITE_QUEEN_UV,
        network::Piece::WhiteKing => WHITE_KING_UV,
        _ => panic!("Unexpected piece"),
    }
}

struct Game {
    is_server:bool,
    chess: layer::Chess,
    moves: Vec<network::Move>,
    piece_sheet: Image,
    quad_mesh: Mesh,
    circle_mesh: Mesh,
    selected: Option<(usize, usize)>,
}

impl Game {
    fn new(ctx: &mut Context, is_server: bool) -> GameResult<Self> {
        Ok(Self {
            is_server,
            chess: layer::Chess::new(),
            selected: None,
            moves: Vec::new(),
            piece_sheet: Image::from_path(ctx, "/pieces.png")?,
            quad_mesh: Mesh::new_rectangle(ctx, graphics::DrawMode::Fill(FillOptions::DEFAULT), Rect::one(), Color::WHITE)?,
            circle_mesh: Mesh::new_circle(ctx, graphics::DrawMode::Fill(FillOptions::DEFAULT), vec2(0.0, 0.0), 1.0, 0.01, Color::WHITE
            )?
        })
    }
}

impl EventHandler for Game {

   fn update(&mut self, ctx: &mut Context) -> GameResult {

        if ctx.mouse.button_just_pressed(event::MouseButton::Right) {
            self.selected = None;
            self.moves.clear();
        }

        if ctx.mouse.button_just_pressed(event::MouseButton::Left) {
            let mouse = ctx.mouse.position();

            let (c, r) = screen_to_tile((mouse.x, mouse.y));
            if let Some((c0, r0)) = self.selected {
                let mv = network::Move { start_x: c0, start_y: r0, end_x: c as usize, end_y: r as usize, promotion: network::Piece::None };

                if (self.is_server) {
                    match self.chess.try_server_move(mv) {
                        Some(state) => {
                            // send over tcp
                        },
                        None => (),
                    };
                } else {
                    match self.chess.try_client_move(mv, tcp) {
                        Some(state) => todo!(),
                        None => todo!(),
                    }
                }

                self.selected = None;
            } else {
                self.selected = Some((c as usize, r as usize));
                self.moves = self.chess.get_moves().into_iter().filter(|m| m.start_x == c && m.start_y == r).collect();
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {

        let mut canvas = graphics::Canvas::from_frame(ctx, BACKGROUND_COLOR);

        let board = self.chess.get_board();
        
        for r in 0..8 {
            for c in 0..8 {
                let (x, y) = tile_to_screen(c, r);
                let dest_rect = Rect::new(x, y, TILE_SIZE_PX as f32, TILE_SIZE_PX as f32);

                let tile_color = if (r + c) % 2 != 0 { TILE_LIGHT_COLOR } else { TILE_DARK_COLOR };
                canvas.draw(&self.quad_mesh, DrawParam::new().dest_rect(dest_rect).color(tile_color).z(0));
                
                let piece = board[r as usize][c as usize];
                if piece != network::Piece::None {
                    let src = get_piece_uv(piece);
                    let dest = dest_rect.point();
                    canvas.draw(&self.piece_sheet, DrawParam::new().src(src).dest(dest).z(3));
                }
            }
        }
        
        let mouse = ctx.mouse.position();
        let (mc, mr) = screen_to_tile((mouse.x, mouse.y));
        let (mx, my) = tile_to_screen(mc, mr);
        let dest = Rect::new(mx, my, TILE_SIZE_PX as f32, TILE_SIZE_PX as f32);
        canvas.draw(&self.quad_mesh, DrawParam::new().dest_rect(dest).color(TILE_HOVER_COLOR).z(2));

        if let Some((sc, sr)) = self.selected {
            let (sx, sy) = tile_to_screen(sc, sr);
            let dest = Rect::new(sx, sy, TILE_SIZE_PX as f32, TILE_SIZE_PX as f32);
            canvas.draw(&self.quad_mesh, DrawParam::new().dest_rect(dest).color(TILE_SELECT_COLOR).z(2));

            for mv in self.moves.iter() {

                // bruh
                // let i = mv.destination;
                // let pos = chess::Pos {
                //     rank: (i as i32 - 21) / 10,
                //     file: (i as i32 - 1) % 10,
                // };
                

                let (mx, my) = tile_to_screen(mv.end_x, mv.end_y);
                let dest = Rect::new(mx, my, TILE_SIZE_PX as f32, TILE_SIZE_PX as f32);
                canvas.draw(&self.quad_mesh, DrawParam::new().dest_rect(dest).color(MOVE_PREVIEW_COLOR).z(1));
            }
        }

        let (hx, hy) = match self.chess.player_to_move() {
            network::Color::White => (2.0 * TURN_HINT_RADIUS, BOARD_SIZE_PX - 2.0 * TURN_HINT_RADIUS),
            network::Color::Black => (2.0 * TURN_HINT_RADIUS, 2.0 * TURN_HINT_RADIUS),
        };

        canvas.draw(&self.circle_mesh, DrawParam::new().scale(vec2(TURN_HINT_RADIUS, TURN_HINT_RADIUS)).dest(vec2(hx, hy)).color(PLAYER_TURN_HINT_COLOR).z(4));

        canvas.finish(ctx)?;
        
        Ok(())
    }

}