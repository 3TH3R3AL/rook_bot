use std::{fmt, usize};

#[allow(dead_code)]
#[derive(PartialEq, Eq, Clone, Copy)]
enum Color {
    White,
    Black,
}
const BOTTOM_SIDE: Color = White;
#[allow(dead_code)]
#[derive(PartialEq, Eq, Clone, Copy)]
enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook { has_moved: bool },
    King { has_moved: bool },
    Queen,
    Empty,
}
#[derive(Clone, PartialEq, Eq)]
struct CoordinateSet {
    x: i32,
    y: i32,
}
struct Direction {
    x: i32,
    y: i32,
}
impl Direction {
    const LEFT: Direction = Direction { x: -1, y: 1 };
}
const BOUNDS: (i32, i32) = (0, 7);
impl CoordinateSet {
    fn out_of_bounds(&self) -> bool {
        self.x < BOUNDS.0 || self.x > BOUNDS.1 || self.y < BOUNDS.0 || self.y > BOUNDS.1
    }
}
impl std::ops::Add<&Direction> for CoordinateSet {
    type Output = CoordinateSet;

    fn add(self, other: &Direction) -> CoordinateSet {
        CoordinateSet {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Add<&Direction> for &CoordinateSet {
    type Output = CoordinateSet;

    fn add(self, other: &Direction) -> CoordinateSet {
        CoordinateSet {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl std::ops::Mul<i32> for Direction {
    type Output = Direction;
    fn mul(self, rhs: i32) -> Self::Output {
        Direction {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

use Color::*;
use MoveType::*;
use PieceType::*;
#[derive(Clone, Copy)]
struct Piece {
    color: Color,
    piece_type: PieceType,
}
#[derive(Clone)]
struct BoardPosition {
    board: [[Piece; 8]; 8],
    en_passante: Option<CoordinateSet>,
    children: Vec<BoardPosition>,
}
#[derive(PartialEq, Eq)]
enum MoveType {
    Standard,
    CaptureOnly,
    NoCapture,
    PawnFirst,
    Promotion,
    Repeat,
    EnPassante,
    Castle,
}
struct ChessMove(MoveType, Direction);
impl Piece {
    fn new(color: Color, piece_type: PieceType) -> Piece {
        Piece { color, piece_type }
    }
    fn character(&self) -> char {
        match (&self.color, &self.piece_type) {
            (White, Pawn) => '♟',
            (White, Knight) => '♞',
            (White, Bishop) => '♝',
            (White, Rook { .. }) => '♜',
            (White, King { .. }) => '♚',
            (White, Queen) => '♛',
            (Black, Pawn) => '♙',
            (Black, Knight) => '♘',
            (Black, Bishop) => '♗',
            (Black, Rook { .. }) => '♖',
            (Black, King { .. }) => '♔',
            (Black, Queen) => '♕',
            (_, Empty) => ' ',
        }
    }
    fn is_empty(&self) -> bool {
        self.piece_type == Empty
    }
    fn forward(&self) -> Direction {
        if self.color == BOTTOM_SIDE {
            Direction { x: 0, y: -1 }
        } else {
            Direction { x: 0, y: 1 }
        }
    }
    fn get_moves(&self) -> Vec<ChessMove> {
        // This is written with the assumption that +y = forward and -y = backward, -x is queen
        // side, +x is king side
        match &self.piece_type {
            Pawn => vec![
                ChessMove(NoCapture, Direction { x: 0, y: 1 }),
                ChessMove(PawnFirst, Direction { x: 0, y: 2 }),
                ChessMove(CaptureOnly, Direction { x: 1, y: 1 }),
                ChessMove(CaptureOnly, Direction { x: -1, y: 1 }),
                ChessMove(EnPassante, Direction { x: 1, y: 1 }),
                ChessMove(EnPassante, Direction { x: -1, y: 1 }),
                ChessMove(Promotion, Direction { x: 0, y: 1 }),
                ChessMove(Promotion, Direction { x: 1, y: 1 }),
                ChessMove(Promotion, Direction { x: -1, y: 1 }),
            ],
            Knight => vec![
                ChessMove(Standard, Direction { x: -1, y: 2 }),
                ChessMove(Standard, Direction { x: 1, y: 2 }),
                ChessMove(Standard, Direction { x: -1, y: -2 }),
                ChessMove(Standard, Direction { x: 1, y: -2 }),
                ChessMove(Standard, Direction { y: -1, x: 2 }),
                ChessMove(Standard, Direction { y: 1, x: 2 }),
                ChessMove(Standard, Direction { y: -1, x: -2 }),
                ChessMove(Standard, Direction { y: 1, x: -2 }),
            ],
            Bishop => vec![
                ChessMove(Repeat, Direction { x: 1, y: 1 }),
                ChessMove(Repeat, Direction { x: -1, y: 1 }),
                ChessMove(Repeat, Direction { x: 1, y: -1 }),
                ChessMove(Repeat, Direction { x: -1, y: -1 }),
            ],
            Rook { .. } => vec![
                ChessMove(Repeat, Direction { x: 1, y: 0 }),
                ChessMove(Repeat, Direction { x: -1, y: 0 }),
                ChessMove(Repeat, Direction { x: 0, y: 1 }),
                ChessMove(Repeat, Direction { x: 0, y: -1 }),
            ],
            Queen => vec![
                ChessMove(Repeat, Direction { x: 1, y: 0 }),
                ChessMove(Repeat, Direction { x: -1, y: 0 }),
                ChessMove(Repeat, Direction { x: 0, y: 1 }),
                ChessMove(Repeat, Direction { x: 0, y: -1 }),
                ChessMove(Repeat, Direction { x: 1, y: 1 }),
                ChessMove(Repeat, Direction { x: -1, y: 1 }),
                ChessMove(Repeat, Direction { x: 1, y: -1 }),
                ChessMove(Repeat, Direction { x: -1, y: -1 }),
            ],
            King { .. } => vec![
                ChessMove(Standard, Direction { x: 1, y: 0 }),
                ChessMove(Standard, Direction { x: -1, y: 0 }),
                ChessMove(Standard, Direction { x: 0, y: 1 }),
                ChessMove(Standard, Direction { x: 0, y: -1 }),
                ChessMove(Standard, Direction { x: 1, y: 1 }),
                ChessMove(Standard, Direction { x: -1, y: 1 }),
                ChessMove(Standard, Direction { x: 1, y: -1 }),
                ChessMove(Standard, Direction { x: -1, y: -1 }),
                ChessMove(Castle, Direction { x: -3, y: 0 }),
                ChessMove(Castle, Direction { x: -2, y: 0 }),
            ],
            Empty => vec![],
        }
    }

    fn point_value(&self) -> i32 {
        match &self.piece_type {
            Pawn => 1,
            Knight => 3,
            Bishop => 3,
            Rook { .. } => 5,
            Queen => 9,
            King { .. } => 10000,
            Empty => 0,
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.character())
    }
}

impl BoardPosition {
    fn new(
        init_arr: [[(Color, PieceType); 8]; 8],
        en_passante: Option<CoordinateSet>,
    ) -> BoardPosition {
        BoardPosition {
            board: init_arr.map(|row| row.map(|cell| Piece::new(cell.0, cell.1))),
            en_passante,
            children: Vec::new(),
        }
    }
    fn get_piece(&self, square: &CoordinateSet) -> &Piece {
        &self.board[square.y as usize][square.x as usize]
    }
    fn set_piece(&mut self, square: &CoordinateSet, piece: Piece) {
        self.board[square.y as usize][square.x as usize] = piece;
    }
    fn clear_square(&mut self, square: &CoordinateSet) {
        self.set_piece(square, Piece::new(White, Empty));
    }
    fn move_arbitrary(
        &self,
        start: &CoordinateSet,
        end: &CoordinateSet,
        cant_capture: bool,
    ) -> Option<BoardPosition> {
        let target = self.get_piece(end);
        if target.piece_type != Empty {
            let moving = self.get_piece(start);
            if cant_capture || moving.color == target.color {
                return None;
            }
        }

        let mut new_board = self.clone();
        new_board.set_piece(end, *self.get_piece(start));
        new_board.clear_square(start);
        Some(new_board)
    }

    fn move_repeat(&mut self, start: CoordinateSet, direction: Direction, repeat: i32) {
        let direction = direction * repeat;
        let target = &start + &direction;
        if self.get_piece(&target).piece_type != Empty || target.out_of_bounds() {
            return;
        }
        let new = self.move_arbitrary(&start, &target, false);
        match new {
            None => (),
            Some(to_add) => {
                self.children.push(to_add);
                self.move_repeat(start, direction, repeat + 1)
            }
        }
    }

    fn push_if_exists(&mut self, to_add: Option<BoardPosition>) {
        match to_add {
            None => (),
            Some(position) => {
                self.children.push(position);
            }
        }
    }
    fn eval_move(&mut self, piece: Piece, coords: &CoordinateSet, move_to_eval: ChessMove) {
        let destination = coords + &move_to_eval.1;
        if destination.out_of_bounds() {
            return;
        }
        let target = self.get_piece(&destination);
        if target.is_empty() {
            if move_to_eval.0 == CaptureOnly {
                return;
            }
        } else {
            if move_to_eval.0 == NoCapture {
                return;
            }
            if move_to_eval.0 == Standard
                || move_to_eval.0 == CaptureOnly && target.color == piece.color
            {
                return;
            }
        }
        match move_to_eval.0 {
            EnPassante => match &self.en_passante {
                None => return,
                Some(enp_coords) => {
                    if enp_coords != coords {
                        return;
                    }
                }
            },
            PawnFirst => {}
            Standard => self.push_if_exists(self.move_arbitrary(coords, &destination, false)),
            _ => {}
        };
    }
    // fn possible_moves(
    //     position: &[[Piece; 8]; 8],
    //     start: CoordinateSet,
    //     mut previous: Vec<BoardPosition>,
    // ) -> Vec<BoardPosition> {
    //     let piece_to_move = position.get_piece(&start);
    //
    //     match piece_to_move.piece_type {
    //         Rook { .. } => position.move_repeat(start, Direction::LEFT, previous, 1),
    //         _ => Vec::new(),
    //     }
    // }
    fn eval_moves(&mut self) {
        for i in 0..self.board.len() {
            for j in 0..self.board[i].len() {
                // self.eval_moves()
                //     self.board,
                //     CoordinateSet {
                //         x: j as i32,
                //         y: i as i32,
                //     },
                //     self.children,
                // );
            }
        }
    }
}

impl fmt::Display for BoardPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let board_string = self
            .board
            .iter()
            .enumerate()
            .fold(String::new(), |str, (i, row)| {
                format!(
                    "{}{}\x1b[40m\n",
                    str,
                    row.iter()
                        .enumerate()
                        .fold(String::new(), |str, (j, piece)| {
                            format!(
                                "{}{} {} ",
                                str,
                                if (j + i) % 2 == 0 {
                                    "\x1b[40m"
                                } else {
                                    "\x1b[41m"
                                },
                                piece.character()
                            )
                        }),
                )
            });
        write!(
            f,
            "{}En Passante: {}",
            board_string,
            if self.en_passante.is_none() {
                "No Eligible Pawns"
            } else {
                "Pawn Eligible"
            }
        )
    }
}

#[rustfmt::skip]
const INITIAL_BOARD: [[(Color,PieceType); 8]; 8] = [
    [(Black,Rook { has_moved: false }),(Black,Knight),(Black,Bishop),(Black,Queen),(Black,King { has_moved: false }),(Black,Bishop),(Black,Knight),(Black,Rook { has_moved: false })],
    [(Black,Pawn),(Black,Pawn),(Black,Pawn),(Black,Pawn),(Black,Pawn),(Black,Pawn),(Black,Pawn),(Black,Pawn)],
    [(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty)],
    [(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty)],
    [(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty)],
    [(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty)],
    [(White,Pawn),(White,Pawn),(White,Pawn),(White,Pawn),(White,Pawn),(White,Pawn),(White,Pawn),(White,Pawn)],
    [(White,Rook { has_moved: false }),(White,Knight),(White,Bishop),(White,Queen),(White,King { has_moved: false }),(White,Bishop),(White,Knight),(White,Rook { has_moved: false })],
];
fn main() {
    let piece = Piece::new(White, Pawn);
    let init_position = match BOTTOM_SIDE {
        White => BoardPosition::new(INITIAL_BOARD, None),
        Black => {
            let mut half_reverse = INITIAL_BOARD.map(|mut row| {
                row.reverse();
                row
            });
            half_reverse.reverse();
            BoardPosition::new(half_reverse, None)
        }
    };

    println!("{}", init_position);
}
