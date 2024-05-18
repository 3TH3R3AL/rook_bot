use core::{panic, time};
use std::cmp::min;
use std::convert::From;
use std::io::stdin;
use std::ops::Not;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, sleep};
use std::time::Instant;
use std::{fmt, usize};
use PieceType::*;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum PieceColor {
    White,
    Black,
}

impl Not for PieceColor {
    type Output = PieceColor;
    fn not(self) -> Self::Output {
        match self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }
}

const BOTTOM_SIDE: PieceColor = PieceColor::White;
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
impl PieceType {
    fn to_promote() -> [PieceType; 4] {
        [Knight, Bishop, Rook { has_moved: true }, Queen]
    }
}
#[rustfmt::skip]
const INITIAL_BOARD: [[(PieceColor,PieceType); 8]; 8] = [
    [(PieceColor::Black,Rook { has_moved: false }),(PieceColor::Black,Knight),(PieceColor::Black,Bishop),(PieceColor::Black,Queen),(PieceColor::Black,King { has_moved: false }),(PieceColor::Black,Bishop),(PieceColor::Black,Knight),(PieceColor::Black,Rook { has_moved: false })],
    [(PieceColor::Black,Pawn),(PieceColor::Black,Pawn),(PieceColor::Black,Pawn),(PieceColor::Black,Pawn),(PieceColor::Black,Pawn),(PieceColor::Black,Pawn),(PieceColor::Black,Pawn),(PieceColor::Black,Pawn)],
    [(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty)],
    [(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty)],
    [(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty)],
    [(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty)],
    [(PieceColor::White,Pawn),(PieceColor::White,Pawn),(PieceColor::White,Pawn),(PieceColor::White,Pawn),(PieceColor::White,Pawn),(PieceColor::White,Pawn),(PieceColor::White,Pawn),(PieceColor::White,Pawn)],
    [(PieceColor::White,Rook { has_moved: false }),(PieceColor::White,Knight),(PieceColor::White,Bishop),(PieceColor::White,Queen),(PieceColor::White,King { has_moved: false }),(PieceColor::White,Bishop),(PieceColor::White,Knight),(PieceColor::White,Rook { has_moved: false })],
];
#[derive(Debug, Clone, PartialEq, Eq)]
struct CoordinateSet {
    x: i32,
    y: i32,
}
impl CoordinateSet {
    fn new(x: i32, y: i32) -> Self {
        CoordinateSet { x, y }
    }
}
#[derive(Debug, Clone)]
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
    color: PieceColor,
    piece_type: PieceType,
}
#[derive(Clone, Debug)]
struct BoardPosition {
    board: Board,
    en_passante: Option<CoordinateSet>,
    children: Vec<BoardPosition>,
    base_white_eval: i32,
    tree_eval: i32,
}
#[derive(Debug, PartialEq, Eq)]
enum MoveType {
    Standard,
    CaptureOnly,
    NoCapture,
    PawnFirst,
    Promotion,
    PromotionCapture,
    Repeat,
    EnPassante,
    Castle,
}
#[derive(Debug)]
struct ChessMove(MoveType, Direction);
impl Piece {
    fn new(color: PieceColor, piece_type: PieceType) -> Piece {
        Piece { color, piece_type }
    }
    fn character(&self) -> char {
        match (&self.color, &self.piece_type) {
            (PieceColor::White, Pawn) => '♟',
            (PieceColor::White, Knight) => '♞',
            (PieceColor::White, Bishop) => '♝',
            (PieceColor::White, Rook { .. }) => '♜',
            (PieceColor::White, King { .. }) => '♚',
            (PieceColor::White, Queen) => '♛',
            (PieceColor::Black, Pawn) => '♙',
            (PieceColor::Black, Knight) => '♘',
            (PieceColor::Black, Bishop) => '♗',
            (PieceColor::Black, Rook { .. }) => '♖',
            (PieceColor::Black, King { .. }) => '♔',
            (PieceColor::Black, Queen) => '♕',
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
            King { has_moved: false } => King { has_moved: true },
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
                    PromotionCapture,
                    Direction {
                        x: 1,
                        y: 1 * self.forward(),
                    },
                ),
                ChessMove(
                    PromotionCapture,
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
                ChessMove(Castle, Direction { x: -2, y: 0 }),
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
    fn image_file_name(&self) -> String {
        use PieceColor::*;
        String::from(match (&self.color, &self.piece_type) {
            (White, Pawn) => "wp.png",
            (White, Knight) => "wn.png",
            (White, Bishop) => "wb.png",
            (White, Rook { .. }) => "wr.png",
            (White, Queen) => "wq.png",
            (White, King { .. }) => "wk.png",
            (Black, Pawn) => "bp.png",
            (Black, Knight) => "bn.png",
            (Black, Bishop) => "bb.png",
            (Black, Rook { .. }) => "br.png",
            (Black, Queen) => "bq.png",
            (Black, King { .. }) => "bk.png",
            (_, Empty) => "",
        })
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.character())
    }
}
impl Default for BoardPosition {
    fn default() -> Self {
        BoardPosition {
            board: INITIAL_BOARD.map(|row| row.map(|cell| Piece::new(cell.0, cell.1))),
            tree_eval: 0,
            en_passante: None,
            base_white_eval: 0,
            children: Vec::new(),
        }
    }
}
impl From<[[(PieceColor, PieceType); 8]; 8]> for BoardPosition {
    fn from(item: [[(PieceColor, PieceType); 8]; 8]) -> BoardPosition {
        BoardPosition {
            board: item.map(|row| row.map(|cell| Piece::new(cell.0, cell.1))),
            ..Default::default()
        }
    }
}
type Board = [[Piece; 8]; 8];

impl BoardPosition {
    fn new(board: Board) -> BoardPosition {
        BoardPosition {
            board,
            ..Default::default()
        }
    }
    fn get_piece(&self, square: &CoordinateSet) -> &Piece {
        debug_assert!(
            !square.out_of_bounds(),
            "get_piece out of bounds: {:?}",
            square
        );
        &self.board[square.y as usize][square.x as usize]
    }
    fn set_piece(&mut self, square: &CoordinateSet, piece: Piece) {
        debug_assert!(
            !square.out_of_bounds(),
            "set_piece out of bounds: {:?}",
            square
        );
        self.board[square.y as usize][square.x as usize] = piece;
    }
    fn clear_square(&mut self, square: &CoordinateSet) {
        debug_assert!(
            !square.out_of_bounds(),
            "clear_square out of bounds: {:?}",
            square
        );
        self.set_piece(square, Piece::new(PieceColor::Black, Empty));
    }
    fn append_child(&mut self, mut new_child: BoardPosition) {
        new_child.base_white_eval = new_child.eval(PieceColor::White);
        self.children.push(new_child);
    }
    fn eval(&self, color_moving: PieceColor) -> i32 {
        self.board.iter().fold(0, |acc, row| {
            acc + row.iter().fold(0, |acc, piece| {
                if piece.color == color_moving {
                    acc + piece.point_value()
                } else {
                    acc - piece.point_value()
                }
            })
        })
    }
    /// How good the position is for color_moving (higher is better) ///
    fn tree_eval(&self, color_moving: PieceColor) -> i32 {
        if self.children.is_empty() {
            return match color_moving {
                PieceColor::White => self.base_white_eval,
                PieceColor::Black => -self.base_white_eval,
            };
        }
        // Best case scenario for moving is worst case for adversary, so minimum and then flip
        -self.children.iter().fold(1000000, |acc, position| {
            min(position.tree_eval(!color_moving), acc)
        })
    }

    fn move_arbitrary(&self, start: &CoordinateSet, end: &CoordinateSet) -> BoardPosition {
        debug_assert!(
            !start.out_of_bounds(),
            "start in move_arbitrary out of bounds: {:?}",
            start
        );
        debug_assert!(
            !end.out_of_bounds(),
            "end in move_arbitrary out of bounds: {:?}",
            end
        );
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
        if destination.out_of_bounds() {
            return;
        }
        let target = self.get_piece(&destination);
        if (!target.is_empty() && target.color == piece.color) || destination.out_of_bounds() {
            return;
        }
        let to_add = self.move_arbitrary(&start, &destination);
        self.append_child(to_add);
        if self.get_piece(&destination).piece_type != Empty {
            return;
        }
        self.move_repeat(start, &direction, repeat + 1)
    }

    // fn push_if_exists(&mut self, to_add: Option<BoardPosition>) {
    //     match to_add {
    //         None => (),
    //         Some(position) => {
    //             self.append_child(position);
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
            if let CaptureOnly | PromotionCapture = move_to_eval.0 {
                return;
            }
        } else {
            if let EnPassante | NoCapture | PawnFirst | Promotion = move_to_eval.0 {
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
                    // Make sure target is available for en passante
                    if *enp_coords != target_pawn {
                        return;
                    }

                    // This should be guaranteed by capture checking
                    debug_assert!(
                        self.get_piece(&destination).is_empty(),
                        "BAD En Passante Target Is Not Empty at {:?}",
                        enp_coords
                    );
                    // Should be guaranteed, but make sure
                    debug_assert!(
                        *self.get_piece(&target_pawn)
                            == Piece {
                                piece_type: Pawn,
                                color: !piece.color,
                            },
                        "BAD En Passante Coords {:?} when moving {:?} Do Not Point To Proper Piece, Point To {:?} when it should be {:?}\nScenario:\n{}",
                        target_pawn,
                        coords,
                        Piece {
                            piece_type: Pawn,
                            color: !piece.color,
                        },
                        *self.get_piece(&target_pawn),
                        self
                    );
                    let mut board = self.move_arbitrary(coords, &destination);
                    board.set_piece(coords, *piece);
                    board.clear_square(&target_pawn);
                    board.clear_square(&coords);
                    self.append_child(board);
                }
            },
            PawnFirst => {
                let row = match piece.color == BOTTOM_SIDE {
                    true => 6,
                    false => 1,
                };
                if coords.y != row {
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
                self.append_child(board);
            }
            Standard | CaptureOnly | NoCapture => {
                self.append_child(self.move_arbitrary(coords, &destination))
            }
            Repeat => self.move_repeat(coords, &move_to_eval.1, 1),
            // May want to consider pulling some of this into another function
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
                let (rook_pos, mid_point) = match move_to_eval.1.x {
                    // Remember king is at position x=4
                    2 => (
                        coords + Direction { x: 3, y: 0 },
                        coords + Direction { x: 1, y: 0 },
                    ),
                    -2 => (
                        coords + Direction { x: -4, y: 0 },
                        coords + Direction { x: -1, y: 0 },
                    ),
                    _ => {
                        panic!("Bad Castling Direction ");
                    }
                };

                debug_assert!(
                    !rook_pos.out_of_bounds(),
                    "rook_pos out of bounds: {:?}, king_pos: {:?}",
                    rook_pos,
                    coords
                );

                let rook = self.get_piece(&rook_pos);
                if rook.piece_type != (Rook { has_moved: false }) {
                    // Don't need to check color because of has_moved
                    return;
                }
                if !self.get_piece(&mid_point).is_empty() {
                    return;
                }
                //Don't need to check second space because it is destination
                let mut test_board = self.move_arbitrary(coords, &mid_point);
                // This method is a little iffy, because it will evaluate moves on the
                // other side, including castling, which could then evaluate white moves,
                // but that should stop after 2 because of has_moved, so it should be fine.
                test_board.eval_moves(!piece.color);
                for board in test_board.children {
                    if *board.get_piece(&mid_point)
                        != (Piece {
                            piece_type: King { has_moved: true },
                            color: piece.color,
                        })
                    {
                        return;
                    }
                }
                let mut final_board = self.move_arbitrary(coords, &destination);
                final_board.set_piece(&mid_point, *rook);
                final_board.clear_square(&rook_pos);
                self.append_child(final_board);
            }

            Promotion | PromotionCapture => {
                let row = match piece.color == BOTTOM_SIDE {
                    true => 1,
                    false => 6,
                };
                if coords.y != row {
                    return;
                }
                let color = piece.color;
                for piece_type in PieceType::to_promote() {
                    let mut new_board = BoardPosition::new(self.board.clone());
                    new_board.clear_square(coords);
                    new_board.set_piece(&destination, Piece { piece_type, color });
                    self.append_child(new_board);
                }
            }
        }
    }
    fn eval_moves(&mut self, player_color: PieceColor) {
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

    fn get_legal_moves_piece(&mut self, target: &CoordinateSet) -> Vec<ChessMove> {
        debug_assert!(!target.out_of_bounds());
        let mut moves: Vec<ChessMove> = Vec::new();
        let piece = self.get_piece(&target);
        debug_assert!(piece.piece_type != Empty);

        let potential_moves = self.get_piece(&target).get_moves();
        for potential_move in potential_moves {
            let old_length = self.children.len();
            self.eval_move(&target, &potential_move);
            if self.children.len() > old_length {
                if potential_move.0 == Repeat {
                    for i in old_length..self.children.len() {
                        moves.push(ChessMove(
                            Standard,
                            &potential_move.1 * (i - old_length + 1) as i32,
                        ));
                    }
                } else {
                    moves.push(potential_move);
                }
            }
        }
        moves
    }

    fn display_with_moves(&mut self, piece: &CoordinateSet) {
        debug_assert!(
            !piece.out_of_bounds(),
            "Error: {:?} is out of bounds in display_with_moves",
            piece
        );
        let moves = self.get_legal_moves_piece(piece);
        let mut move_squares = [[0; 8]; 8];
        moves.into_iter().for_each(|chess_move| {
            let coords = piece.clone() + chess_move.1;
            move_squares[coords.y as usize][coords.x as usize] = 1;
        });
        let board_string = self
            .board
            .iter()
            .enumerate()
            .fold(String::new(), |str, (i, row)| {
                format!(
                    "{}{} {}\x1b[40m\n",
                    str,
                    i,
                    row.iter()
                        .enumerate()
                        .fold(String::new(), |str, (j, piece)| {
                            format!(
                                "{}{} {} ",
                                str,
                                if move_squares[i][j] == 1 {
                                    "\x1b[42m"
                                } else if (j + i) % 2 == 0 {
                                    "\x1b[40m"
                                } else {
                                    "\x1b[41m"
                                },
                                piece.character()
                            )
                        }),
                )
            });
        println!(
            "   a  b  c  d  e  f  g  h\n{}En Passante: {}",
            board_string,
            if self.en_passante.is_none() {
                "No Eligible Pawns"
            } else {
                "Pawn Eligible"
            }
        )
    }
    fn move_piece(
        &mut self,
        from: CoordinateSet,
        to: CoordinateSet,
    ) -> Result<BoardPosition, String> {
        let moves = self.get_legal_moves_piece(&from);
        let potential_move = moves.into_iter().find(|chess_move| {
            let coords = from.clone() + chess_move.1.clone();
            coords == to
        });
        match potential_move {
            Some(chosen_move) => {
                self.children.clear();
                self.eval_move(&from, &chosen_move);
                Ok(self.children.remove(0))
            }
            None => Err(String::from("Illegal Move, try again")),
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
                    "{}{} {}\x1b[40m\n",
                    str,
                    i,
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
            "   a  b  c  d  e  f  g  h\n{}En Passante: {}",
            board_string,
            if self.en_passante.is_none() {
                "No Eligible Pawns"
            } else {
                "Pawn Eligible"
            }
        )
    }
}

fn expand_tree(
    root: &mut BoardPosition,
    progress: &mut Vec<usize>,
    min_depth: usize,
    max_depth: usize,
    initial_turn: PieceColor,
) {
    let depth = progress.len();
    if depth > max_depth {
        // This is so that the bot doesn't infinite loop if it's gone to max depth
        println!("Bot surpassed maximum");
        sleep(time::Duration::from_millis(100));
        return;
    }
    let mut current: &mut BoardPosition = root;
    if progress[min_depth] == 1 {
        progress.fill(0);
        progress.push(0);
        return expand_tree(root, progress, min_depth, max_depth, initial_turn);
    }
    for i in min_depth + 1..progress.len() {
        let p = progress[i];
        debug_assert!(p <= current.children.len());

        if current.children.len() == p {
            progress[i] = 0;
            progress[i - 1] += 1;
            return expand_tree(root, progress, min_depth, max_depth, initial_turn);
        }

        current = &mut current.children[p];
    }
    current.eval_moves(if depth % 2 == 1 {
        initial_turn
    } else {
        !initial_turn
    });
    progress[depth - 1] += 1;
}

#[derive(Clone, Copy)]
enum MessageToBot {
    Stop,
    Move(Board),
}
#[derive(Debug)]
enum MessageToMain {
    Error(String),
    Move(BoardPosition),
}

fn run_bot(
    bot_out: Sender<MessageToMain>,
    bot_in: Receiver<MessageToBot>,
    initial_position: Board,
    bot_color: PieceColor,
    initial_turn: PieceColor,
) {
    let mut position = BoardPosition::new(initial_position);
    let min_depth = 0;
    let max_depth = 5;
    let mut evaluated = 0;
    let mut progress = vec![0];
    let mut old = Instant::now();
    loop {
        match bot_in.try_recv() {
            Ok(message) => match message {
                MessageToBot::Move(board) => {
                    // let move_index = position.children.iter().position(|a| {
                    //     a.board.iter().zip(board).fold(true||)
                    // });
                    let move_index = position.children.iter().position(|a| a.board == board);
                    match move_index {
                        None => {
                            bot_out
                                .send(MessageToMain::Error(String::from("Illegal Move")))
                                .unwrap();
                        }
                        Some(index) => {
                            position = position.children.remove(index);
                            let current_progress = progress.remove(0);
                            if current_progress != index {
                                progress.fill(0);
                            }
                            position
                                .children
                                .iter_mut()
                                .for_each(|child| child.tree_eval = child.tree_eval(!bot_color));

                            let (move_correct, current_eval) = position
                                .children
                                .iter()
                                .enumerate()
                                .fold((0 as usize, 100000), |current, new| {
                                    if new.1.tree_eval < current.1 {
                                        (new.0, new.1.tree_eval)
                                    } else {
                                        current
                                    }
                                });

                            println!("Current Eval: {}", current_eval);
                            position = position.children.swap_remove(move_correct);
                            let current_progress = progress.remove(0);
                            if current_progress != index {
                                progress.fill(0);
                            }
                            bot_out
                                .send(MessageToMain::Move(BoardPosition {
                                    children: Vec::new(),
                                    board: position.board.clone(),
                                    en_passante: position.en_passante.clone(),
                                    ..position
                                }))
                                .unwrap();
                        }
                    }
                }
                MessageToBot::Stop => {
                    return;
                }
            },
            Err(e) => match e {
                mpsc::TryRecvError::Empty => {
                    evaluated += 1;
                    if evaluated % 100000 == 0 {
                        let secs_taken = old.elapsed().as_secs_f64();
                        println!(
                            "Evaluated 100k moves in {} seconds ({} moves/second)",
                            secs_taken,
                            100000.0 / secs_taken
                        );
                        old = Instant::now();
                    }
                    expand_tree(
                        &mut position,
                        &mut progress,
                        min_depth,
                        max_depth,
                        initial_turn,
                    )
                }
                mpsc::TryRecvError::Disconnected => {
                    return;
                }
            },
        }
    }
}

// const INITIAL_BOARD: [[(Color,PieceType); 8]; 8] = [
//     [(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,King { has_moved: false}),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty)],
//     [(PieceColor::White,Pawn),(PieceColor::Black,Pawn),(PieceColor::Black,Pawn),(PieceColor::Black,Pawn),(PieceColor::Black,Pawn),(PieceColor::Black,Pawn),(PieceColor::Black,Pawn),(PieceColor::Black,Pawn)],
//     [(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty)],
//     [(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty)],
//     [(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty)],
//     [(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty)],
//     [(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::Black,Empty)],
//     [(PieceColor::White,Rook { has_moved: false }),(PieceColor::Black,Empty),(PieceColor::Black,Empty),(PieceColor::White,Empty),(PieceColor::White,King { has_moved: false }),(PieceColor::White,Empty),(PieceColor::White,Empty),(PieceColor::White,Rook { has_moved: false })],
// ];
fn convert_notation_to_coords(notation: String) -> Result<CoordinateSet, String> {
    let notation = String::from(notation.trim());
    let mut chars = notation.chars();

    let x_chars = vec!['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
    let x: i32 = match chars.next() {
        Some(char) => match x_chars.iter().position(|&c| char == c) {
            Some(index) => index as i32,
            None => {
                return Err(String::from("Invalid X Position"));
            }
        },
        None => {
            return Err(String::from("No Input"));
        }
    };

    let y = match chars.next() {
        Some(char) => match char.to_digit(10) {
            Some(index) => index as i32,
            None => {
                return Err(String::from("Invalid Y Position"));
            }
        },
        None => {
            return Err(String::from("Invalid Y Position"));
        }
    };

    Ok(CoordinateSet::new(x, y))
}
fn command_line_ui(
    main_in: Receiver<MessageToMain>,
    main_out: Sender<MessageToBot>,
    player_color: PieceColor,
    mut current_position: BoardPosition,
) {
    loop {
        println!("{}", current_position);
        println!("Please enter desired piece: ",);
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read line");

        let piece_coords = match convert_notation_to_coords(input) {
            Ok(num) => num,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };

        let piece = current_position.get_piece(&piece_coords);
        if piece.piece_type == Empty || piece.color != player_color {
            println!("You don't have a piece there");
            continue;
        }
        current_position.display_with_moves(&piece_coords);

        println!("Please enter desired move: ",);
        input = String::new();
        stdin().read_line(&mut input).expect("Failed to read line");
        let target = match convert_notation_to_coords(input) {
            Ok(num) => num,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };

        let moves = current_position.get_legal_moves_piece(&piece_coords);
        let potential_move = moves.into_iter().find(|chess_move| {
            let coords = piece_coords.clone() + chess_move.1.clone();
            coords == target
        });
        match potential_move {
            Some(chosen_move) => {
                current_position.children.clear();
                current_position.eval_move(&piece_coords, &chosen_move);
                current_position = current_position.children.remove(0);
                main_out
                    .send(MessageToBot::Move(current_position.board))
                    .unwrap();
            }
            None => {
                println!("Illegal Move, try again");
                continue;
            }
        }
        let incoming = main_in.recv();
        match incoming {
            Ok(message) => match message {
                MessageToMain::Move(new_position) => {
                    current_position = new_position;
                }
                MessageToMain::Error(e) => {
                    println!("Bot received ERROR:\n{}", e);
                    break;
                }
            },
            Err(e) => {
                println!("Bot failed to receive:\n{}", e);
                break;
            }
        }
    }
    main_out.send(MessageToBot::Stop).unwrap();
}

use macroquad::prelude::*;
const BOARD_SIZE: i32 = 8;
const PADDING_SIZE: f32 = 50.0;
async fn graphical_ui(
    main_in: Receiver<MessageToMain>,
    main_out: Sender<MessageToBot>,
    player_color: PieceColor,
    mut current_position: BoardPosition,
) {
    let mut dragging_piece: Option<(CoordinateSet, Piece)> = None;
    let mut mouse_offset = vec2(0.0, 0.0);
    loop {
        let square_size: f32 = (min(screen_width() as i32, screen_height() as i32) as f32
            - PADDING_SIZE * 2.0)
            / BOARD_SIZE as f32;
        clear_background(WHITE);
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                let color = if (i + j) % 2 == 0 {
                    color_u8!(235, 236, 208, 255)
                } else {
                    color_u8!(119, 149, 86, 255)
                };
                draw_rectangle(
                    i as f32 * square_size + PADDING_SIZE,
                    j as f32 * square_size + PADDING_SIZE,
                    square_size,
                    square_size,
                    color,
                );
                let piece = current_position.get_piece(&CoordinateSet::new(i, j));
                if piece.piece_type == Empty {
                    continue;
                }
                let mut base_path = String::from("assets/");
                base_path.push_str(&piece.image_file_name());
                let texture = load_texture(&base_path).await.unwrap();
                if let Some((drag_coord, _)) = &dragging_piece {
                    if drag_coord.x == i && drag_coord.y == j {
                        continue;
                    }
                }
                draw_texture_ex(
                    &texture,
                    PADDING_SIZE + (i as f32 * square_size),
                    PADDING_SIZE + (j as f32 * square_size),
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(square_size, square_size)),
                        ..Default::default()
                    },
                );
            }
        }
        if let Some((from, piece)) = &dragging_piece {
            let mut base_path = String::from("assets/");
            base_path.push_str(&piece.image_file_name());
            let texture = load_texture(&base_path).await.unwrap();
            let mouse_position = mouse_position();
            draw_texture_ex(
                &texture,
                mouse_position.0 + mouse_offset.x - square_size / 2.0,
                mouse_position.1 + mouse_offset.y - square_size / 2.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(square_size, square_size)),
                    ..Default::default()
                },
            );
            let moves = current_position.get_legal_moves_piece(from);
            let circle_color = color_u8!(100, 100, 100, 100);
            for chess_move in moves {
                let target = from + chess_move.1;
                draw_circle(
                    (target.x as f32 + 0.5) * square_size + PADDING_SIZE,
                    (target.y as f32 + 0.5) * square_size + PADDING_SIZE,
                    square_size / 6.0,
                    circle_color,
                );
            }
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            let mouse_position = mouse_position();
            let i = ((mouse_position.0 - PADDING_SIZE) / square_size).floor() as i32;
            let j = ((mouse_position.1 - PADDING_SIZE) / square_size).floor() as i32;
            if i >= 0 && i < BOARD_SIZE && j >= 0 && j < BOARD_SIZE {
                let coord = CoordinateSet::new(i, j);
                let piece = current_position.get_piece(&coord);
                if piece.piece_type != Empty {
                    dragging_piece = Some((coord, *piece));
                    mouse_offset = vec2(
                        (mouse_position.0 - PADDING_SIZE) % square_size - square_size / 2.0,
                        (mouse_position.1 - PADDING_SIZE) % square_size - square_size / 2.0,
                    );
                }
            }
        }

        if is_mouse_button_released(MouseButton::Left) {
            if let Some((from, piece)) = dragging_piece.take() {
                let mouse_position = mouse_position();
                let i = ((mouse_position.0 - PADDING_SIZE) / square_size).floor() as i32;
                let j = ((mouse_position.1 - PADDING_SIZE) / square_size).floor() as i32;
                if i >= 0 && i < BOARD_SIZE && j >= 0 && j < BOARD_SIZE {
                    let to = CoordinateSet::new(i, j);
                    match current_position.move_piece(from, to) {
                        Ok(new_board) => {
                            current_position = new_board;

                            main_out
                                .send(MessageToBot::Move(current_position.board))
                                .unwrap();
                            let incoming = main_in.recv();
                            match incoming {
                                Ok(message) => match message {
                                    MessageToMain::Move(new_position) => {
                                        current_position = new_position;
                                    }
                                    MessageToMain::Error(e) => {
                                        println!("Bot received ERROR:\n{}", e);
                                        break;
                                    }
                                },
                                Err(e) => {
                                    println!("Bot failed to receive:\n{}", e);
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        next_frame().await;
    }

    // main_out.send(MessageToBot::Stop).unwrap();
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let mut init_position = match BOTTOM_SIDE {
        PieceColor::White => BoardPosition::from(INITIAL_BOARD),
        PieceColor::Black => {
            let mut half_reverse = INITIAL_BOARD.map(|mut row| {
                row.reverse();
                row
            });
            half_reverse.reverse();
            BoardPosition::from(half_reverse)
        }
    };

    println!("Base Eval: {}", init_position.eval(PieceColor::White));
    let (main_out, bot_in) = mpsc::channel();
    let (bot_out, main_in) = mpsc::channel();

    let bot = thread::spawn(move || {
        run_bot(
            bot_out,
            bot_in,
            init_position.board.clone(),
            PieceColor::Black,
            PieceColor::White,
        )
    });

    graphical_ui(main_in, main_out, PieceColor::White, init_position).await;
    println!("stopping bot");
    bot.join().unwrap();
}
