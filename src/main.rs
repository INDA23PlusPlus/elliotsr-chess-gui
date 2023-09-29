use chess::Tile;
use ggez::conf::{WindowSetup, WindowMode};
use ggez::glam::{vec2, Vec2};
use ggez::{Context, ContextBuilder, GameResult};
use ggez::graphics::{self, Color, Image, Mesh, FillOptions, Rect, DrawParam, Text};
use ggez::event::{self, EventHandler};

use api as chess;

fn main() -> GameResult {
    let (mut ctx, event_loop) = ContextBuilder::new("chess", "elliotsr")
        .window_setup(WindowSetup::default().title("chess - by elliotsr"))
        .window_mode(WindowMode::default().dimensions(BOARD_SIZE_PX as f32, BOARD_SIZE_PX as f32))
        .add_resource_path("res")
        .build()?;

    let game = Game::new(&mut ctx)?;
    
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

fn tile_to_screen(col: u32, row: u32) -> (f32, f32) {
    let x = col as f32 * TILE_SIZE_PX;
    let y = (7 - row) as f32 * TILE_SIZE_PX;
    (x, y)
}

fn screen_to_tile((x, y): (f32, f32)) -> (u32, u32) {
    let col = (x / TILE_SIZE_PX) as u32;
    let row = 7 - (y / TILE_SIZE_PX) as u32;
    (col, row)
}

fn get_piece_uv(tile: &chess::Tile) -> Rect {
    match tile {

        Tile::Pawn(chess::Color::White) => WHITE_PAWN_UV,
        Tile::Bishop(chess::Color::White) => WHITE_BISHOP_UV,
        Tile::Knight(chess::Color::White) => WHITE_KNIGHT_UV,
        Tile::Rook(chess::Color::White) => WHITE_ROOK_UV,
        Tile::Queen(chess::Color::White) => WHITE_QUEEN_UV,
        Tile::King(chess::Color::White) => WHITE_KING_UV,

        Tile::Pawn(chess::Color::Black) => BLACK_PAWN_UV,
        Tile::Bishop(chess::Color::Black) => BLACK_BISHOP_UV,
        Tile::Knight(chess::Color::Black) => BLACK_KNIGHT_UV,
        Tile::Rook(chess::Color::Black) => BLACK_ROOK_UV,
        Tile::Queen(chess::Color::Black) => BLACK_QUEEN_UV,
        Tile::King(chess::Color::Black) => BLACK_KING_UV,

        _ => panic!("Unexpected piece"),
    }
}

struct Game {
    chess: chess::Game,
    piece_sheet: Image,
    quad_mesh: Mesh,
    circle_mesh: Mesh,
    selected: Option<(usize, usize)>,
    moves: Vec<chess::Ply>
}

impl Game {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        Ok(Self {
            chess: chess::default_game(),
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
                let from = chess::Pos { file: c0 as i32, rank: r0 as i32 };
                let to = chess::Pos { file: c as i32, rank: r as i32 };
                _ = self.chess.ply(from, to);

                if self.chess.is_checkmate() {
                    ctx.request_quit();
                }

                self.selected = None;
            } else {
                self.selected = Some((c as usize, r as usize));
                let pos = chess::Pos { file: c as i32, rank: r as i32 };
                self.moves = self.chess.get_plys_from_pos(pos).into_iter().map(|&m| m).collect();
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, BACKGROUND_COLOR);
        
        let board = self.chess.get_board_2d();
        for r in 0..8 {
            for c in 0..8 {
                let (x, y) = tile_to_screen(c, r);
                let dest_rect = Rect::new(x, y, TILE_SIZE_PX as f32, TILE_SIZE_PX as f32);

                let tile_color = if (r + c) % 2 != 0 { TILE_LIGHT_COLOR } else { TILE_DARK_COLOR };
                canvas.draw(&self.quad_mesh, DrawParam::new().dest_rect(dest_rect).color(tile_color).z(0));
                
                let piece = board[r as usize][c as usize];
                if piece != &chess::Tile::Empty {
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
            let (sx, sy) = tile_to_screen(sc as u32, sr as u32);
            let dest = Rect::new(sx, sy, TILE_SIZE_PX as f32, TILE_SIZE_PX as f32);
            canvas.draw(&self.quad_mesh, DrawParam::new().dest_rect(dest).color(TILE_SELECT_COLOR).z(2));

            for mv in self.moves.iter() {
                // bruh
                let i = mv.destination;
                let pos = chess::Pos {
                    rank: (i as i32 - 21) / 10,
                    file: (i as i32 - 1) % 10,
                };
                let (mx, my) = tile_to_screen(pos.file as u32, pos.rank as u32);
                let dest = Rect::new(mx, my, TILE_SIZE_PX as f32, TILE_SIZE_PX as f32);
                canvas.draw(&self.quad_mesh, DrawParam::new().dest_rect(dest).color(MOVE_PREVIEW_COLOR).z(1));
            }
        }

        let (hx, hy) = match self.chess.get_player() {
            chess::Color::White => (2.0 * TURN_HINT_RADIUS, BOARD_SIZE_PX - 2.0 * TURN_HINT_RADIUS),
            chess::Color::Black => (2.0 * TURN_HINT_RADIUS, 2.0 * TURN_HINT_RADIUS),
        };

        canvas.draw(&self.circle_mesh, DrawParam::new().scale(vec2(TURN_HINT_RADIUS, TURN_HINT_RADIUS)).dest(vec2(hx, hy)).color(PLAYER_TURN_HINT_COLOR).z(4));

        canvas.finish(ctx)?;
        
        Ok(())
    }

}