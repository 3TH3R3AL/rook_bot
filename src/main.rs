use core::panic;
use std::convert::From;
use std::{fmt, usize};
use Color::*;
use PieceType::*;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Color {
    White,
    Black,
}
const BOTTOM_SIDE: Color = White;
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook { has_moved: bool },
    King { has_moved: bool },
    Queen,
    Empty,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct CoordinateSet {
    x: i32,
    y: i32,
}
#[derive(Debug)]
struct Direction {
    x: i32,
    y: i32,
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

impl std::ops::Add<Direction> for CoordinateSet {
    type Output = CoordinateSet;

    fn add(self, other: Direction) -> CoordinateSet {
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

impl std::ops::Add<Direction> for &CoordinateSet {
    type Output = CoordinateSet;

    fn add(self, other: Direction) -> CoordinateSet {
        CoordinateSet {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl std::ops::Mul<i32> for &Direction {
    type Output = Direction;
    fn mul(self, rhs: i32) -> Self::Output {
        Direction {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

use MoveType::*;
#[derive(Debug, PartialEq, Clone, Copy)]
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
#[derive(Debug, PartialEq, Eq)]
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
#[derive(Debug)]
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
    fn forward(&self) -> i32 {
        if self.color == BOTTOM_SIDE {
            -1
        } else {
            1
        }
    }
    fn set_moved(&mut self) {
        self.piece_type = match self.piece_type {
            King { has_moved: false } => Rook { has_moved: true },
            Rook { has_moved: false } => Rook { has_moved: true },
            _ => self.piece_type,
        }
    }
    fn get_moves(&self) -> Vec<ChessMove> {
        // This is written with the assumption that -x is queen side, +x is king side
        match &self.piece_type {
            Pawn => vec![
                ChessMove(
                    NoCapture,
                    Direction {
                        x: 0,
                        y: 1 * self.forward(),
                    },
                ),
                ChessMove(
                    PawnFirst,
                    Direction {
                        x: 0,
                        y: 2 * self.forward(),
                    },
                ),
                ChessMove(
                    CaptureOnly,
                    Direction {
                        x: 1,
                        y: 1 * self.forward(),
                    },
                ),
                ChessMove(
                    CaptureOnly,
                    Direction {
                        x: -1,
                        y: 1 * self.forward(),
                    },
                ),
                ChessMove(
                    EnPassante,
                    Direction {
                        x: 1,
                        y: 1 * self.forward(),
                    },
                ),
                ChessMove(
                    EnPassante,
                    Direction {
                        x: -1,
                        y: 1 * self.forward(),
                    },
                ),
                ChessMove(
                    Promotion,
                    Direction {
                        x: 0,
                        y: 1 * self.forward(),
                    },
                ),
                ChessMove(
                    Promotion,
                    Direction {
                        x: 1,
                        y: 1 * self.forward(),
                    },
                ),
                ChessMove(
                    Promotion,
                    Direction {
                        x: -1,
                        y: 1 * self.forward(),
                    },
                ),
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
                ChessMove(Castle, Direction { x: 2, y: 0 }),
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
impl From<[[(Color, PieceType); 8]; 8]> for BoardPosition {
    fn from(item: [[(Color, PieceType); 8]; 8]) -> BoardPosition {
        BoardPosition {
            board: item.map(|row| row.map(|cell| Piece::new(cell.0, cell.1))),
            en_passante: None,
            children: Vec::new(),
        }
    }
}

impl BoardPosition {
    fn new(board: [[Piece; 8]; 8]) -> BoardPosition {
        BoardPosition {
            board,
            en_passante: None,
            children: Vec::new(),
        }
    }
    fn get_piece(&self, square: &CoordinateSet) -> &Piece {
        debug_assert!(!square.out_of_bounds(), "get_piece out of bounds");
        &self.board[square.y as usize][square.x as usize]
    }
    fn set_piece(&mut self, square: &CoordinateSet, piece: Piece) {
        debug_assert!(!square.out_of_bounds(), "set_piece out of bounds");
        self.board[square.y as usize][square.x as usize] = piece;
    }
    fn clear_square(&mut self, square: &CoordinateSet) {
        debug_assert!(!square.out_of_bounds(), "clear_square out of bounds");
        self.set_piece(square, Piece::new(White, Empty));
    }

    fn move_arbitrary(&self, start: &CoordinateSet, end: &CoordinateSet) -> BoardPosition {
        debug_assert!(
            !start.out_of_bounds(),
            "start in move_arbitrary out of bounds"
        );
        debug_assert!(!end.out_of_bounds(), "end in move_arbitrary out of bounds");
        let mut new_board = BoardPosition::new(self.board.clone());
        let mut new_piece = *self.get_piece(start);
        new_piece.set_moved();
        new_board.set_piece(end, new_piece);
        new_board.clear_square(start);
        new_board
    }

    fn move_repeat(&mut self, start: &CoordinateSet, direction: &Direction, repeat: i32) {
        let piece = self.get_piece(start);
        let destination = start + direction * repeat;
        let target = self.get_piece(&destination);
        if (!target.is_empty() && target.color == piece.color) || destination.out_of_bounds() {
            return;
        }
        let to_add = self.move_arbitrary(&start, &destination);
        self.children.push(to_add);
        if self.get_piece(&destination).piece_type != Empty {
            return;
        }
        self.move_repeat(start, &direction, repeat + 1)
    }

    // fn push_if_exists(&mut self, to_add: Option<BoardPosition>) {
    //     match to_add {
    //         None => (),
    //         Some(position) => {
    //             self.children.push(position);
    //         }
    //     }
    // }
    fn eval_move(&mut self, coords: &CoordinateSet, move_to_eval: &ChessMove) {
        // dbg!(move_to_eval);
        let piece = self.get_piece(coords);
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
            if let NoCapture | PawnFirst = move_to_eval.0 {
                return;
            }
            if target.color == piece.color {
                return;
            }
        }
        // Guarantees going into match: destination is in bounds and capture rules are checked
        match move_to_eval.0 {
            EnPassante => match &self.en_passante {
                None => (),
                Some(enp_coords) => {
                    let mut target_pawn = destination.clone();
                    target_pawn.y -= piece.forward();
                    if *enp_coords != target_pawn {
                        ()
                    }
                    debug_assert!(
                        self.get_piece(&destination).is_empty(),
                        "BAD En Passante Target Is Not Empty at {:?}",
                        enp_coords
                    );
                    debug_assert!(
                        *self.get_piece(&target_pawn)
                            != Piece {
                                piece_type: Pawn,
                                color: piece.color
                            },
                        "BAD En Passante Coords Do Not Point To Proper Piece, Point To {:?}",
                        *self.get_piece(&target_pawn)
                    );
                    let mut board = self.move_arbitrary(coords, &destination);
                    board.set_piece(coords, *piece);
                    board.clear_square(&target_pawn);
                    board.clear_square(&coords);
                }
            },
            PawnFirst => {
                let row = match piece.color == BOTTOM_SIDE {
                    true => 6,
                    false => 1,
                };
                if destination.y != row {
                    return;
                } // In correct row
                if !self
                    .get_piece(&CoordinateSet {
                        x: destination.x,
                        y: destination.y - piece.forward(),
                    })
                    .is_empty()
                {
                    return;
                } // Not moving through other piece
                let mut board = self.move_arbitrary(coords, &destination);
                board.en_passante = Some(destination);
                self.children.push(board);
            }
            Standard | CaptureOnly | NoCapture => self
                .children
                .push(self.move_arbitrary(coords, &destination)),
            Repeat => self.move_repeat(coords, &move_to_eval.1, 1),
            Castle => {
                match piece.piece_type {
                    King { has_moved } => {
                        if has_moved {
                            return;
                        }
                    }
                    _ => {
                        panic!("Non King Castling Attempted");
                    }
                };
                match move_to_eval.1.x {
                    // Remember king is at position x=4
                    2 => {
                        let right_rook = self.get_piece(&(coords + Direction { x: 3, y: 0 }));
                        if right_rook.piece_type != Rook { has_moved: false } {
                            return;
                        }
                        ()

                    }
                    -3 => {

                        if right.piece_type != Rook { has_moved: false } {
                            return;
                        }
                    }
                    _ => {
                        panic!("Bad Castling Direction ");
                    }
                }
            }
        }
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
    fn eval_moves(&mut self, player_color: Color) {
        for i in 0..self.board.len() {
            for j in 0..self.board[i].len() {
                let target = CoordinateSet {
                    x: j as i32,
                    y: i as i32,
                };
                let piece = self.get_piece(&target);
                if piece.color != player_color || piece.piece_type == Empty {
                    continue;
                }
                self.board[i][j]
                    .get_moves()
                    .iter()
                    .for_each(|move_to_eval| self.eval_move(&target, &move_to_eval));
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
    let mut init_position = match BOTTOM_SIDE {
        White => BoardPosition::from(INITIAL_BOARD),
        Black => {
            let mut half_reverse = INITIAL_BOARD.map(|mut row| {
                row.reverse();
                row
            });
            half_reverse.reverse();
            BoardPosition::from(half_reverse)
        }
    };
    init_position.eval_moves(White);
    for position in init_position.children {
        println!("{}", position);
    }
}
