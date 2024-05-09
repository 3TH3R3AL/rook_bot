use core::{panic, time};
use std::cmp::min;
use std::convert::From;
use std::ops::Not;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, sleep};
use std::{fmt, usize};
use Color::*;
use PieceType::*;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Color {
    White,
    Black,
}

impl Not for Color {
    type Output = Color;
    fn not(self) -> Self::Output {
        match self {
            White => Black,
            Black => White,
        }
    }
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
impl PieceType {
    fn to_promote() -> [PieceType; 4] {
        [Knight, Bishop, Rook { has_moved: true }, Queen]
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
impl From<[[(Color, PieceType); 8]; 8]> for BoardPosition {
    fn from(item: [[(Color, PieceType); 8]; 8]) -> BoardPosition {
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
        self.set_piece(square, Piece::new(Black, Empty));
    }
    fn append_child(&mut self, mut new_child: BoardPosition) {
        new_child.base_white_eval = new_child.eval(White);
        self.children.push(new_child);
    }
    fn eval(&self, color_moving: Color) -> i32 {
        self.board.iter().fold(0, |acc, row| {
            acc + row.iter().fold(0, |acc, piece| {
                if piece.color == color_moving {
                    acc + piece.point_value()
                } else {
                    acc + piece.point_value()
                }
            })
        })
    }
    /// How good the position is for color_moving (higher is better) ///
    fn tree_eval(&self, color_moving: Color) -> i32 {
        if self.children.is_empty() {
            return match color_moving {
                White => self.base_white_eval,
                Black => -self.base_white_eval,
            };
        }
        // Best case scenario for moving is worst case for adversary, so minimum and then flip
        -self.children.iter().fold(-1000000, |acc, position| {
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

    fn get_legal_moves(&mut self, player_color: Color) -> Vec<ChessMove> {
        let mut moves: Vec<ChessMove> = Vec::new();
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

                let potential_moves = self.board[i][j].get_moves();
                for potential_move in potential_moves {
                    let current_length = self.children.len();
                    self.eval_move(&target, &potential_move);
                    if self.children.len() > current_length {
                        moves.push(potential_move);
                    }
                }
            }
        }
        moves
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

fn expand_tree(
    root: &mut BoardPosition,
    progress: &mut Vec<usize>,
    min_depth: usize,
    max_depth: usize,
    initial_turn: Color,
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
    println!("evaled {:?}", progress);
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
    Move(Board),
}

fn run_bot(
    bot_out: Sender<MessageToMain>,
    bot_in: Receiver<MessageToBot>,
    initial_position: Board,
    bot_color: Color,
    initial_turn: Color,
) {
    let mut position = BoardPosition::new(initial_position);
    let min_depth = 0;
    let max_depth = 5;
    let mut progress = vec![0];
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
                            position
                                .children
                                .iter_mut()
                                .for_each(|child| child.tree_eval = child.tree_eval(!bot_color));

                            let (move_correct, _) = position.children.iter().enumerate().fold(
                                (0 as usize, 100000),
                                |current, new| {
                                    if new.1.tree_eval < current.1 {
                                        (new.0, new.1.tree_eval)
                                    } else {
                                        current
                                    }
                                },
                            );
                            position = position.children.swap_remove(move_correct);
                            bot_out.send(MessageToMain::Move(position.board)).unwrap();
                        }
                    }
                }
                MessageToBot::Stop => {
                    return;
                }
            },
            Err(e) => match e {
                mpsc::TryRecvError::Empty => expand_tree(
                    &mut position,
                    &mut progress,
                    min_depth,
                    max_depth,
                    initial_turn,
                ),
                mpsc::TryRecvError::Disconnected => {
                    break;
                }
            },
        }
    }
}

// const INITIAL_BOARD: [[(Color,PieceType); 8]; 8] = [
//     [(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,King { has_moved: false}),(Black,Empty),(Black,Empty),(Black,Empty)],
//     [(White,Pawn),(Black,Pawn),(Black,Pawn),(Black,Pawn),(Black,Pawn),(Black,Pawn),(Black,Pawn),(Black,Pawn)],
//     [(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty)],
//     [(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty)],
//     [(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty)],
//     [(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty)],
//     [(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty)],
//     [(White,Rook { has_moved: false }),(Black,Empty),(Black,Empty),(White,Empty),(White,King { has_moved: false }),(White,Empty),(White,Empty),(White,Rook { has_moved: false })],
// ];

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

    let (main_out, bot_in) = mpsc::channel();
    let (bot_out, main_in) = mpsc::channel();

    let bot =
        thread::spawn(move || run_bot(bot_out, bot_in, init_position.board.clone(), Black, White));
    sleep(time::Duration::from_millis(1000));

    #[rustfmt::skip]
    let after_move = BoardPosition::from([
        [(Black,Rook { has_moved: false }),(Black,Knight),(Black,Bishop),(Black,Queen),(Black,King { has_moved: false }),(Black,Bishop),(Black,Knight),(Black,Rook { has_moved: false })],
        [(Black,Pawn),(Black,Pawn),(Black,Pawn),(Black,Pawn),(Black,Pawn),(Black,Pawn),(Black,Pawn),(Black,Pawn)],
        [(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty)],
        [(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty)],
        [(Black,Empty),(Black,Empty),(Black,Empty),(White,Pawn),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty)],
        [(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty),(Black,Empty)],
        [(White,Pawn),(White,Pawn),(White,Pawn),(Black,Empty),(White,Pawn),(White,Pawn),(White,Pawn),(White,Pawn)],
        [(White,Rook { has_moved: false }),(White,Knight),(White,Bishop),(White,Queen),(White,King { has_moved: false }),(White,Bishop),(White,Knight),(White,Rook { has_moved: false })],
    ]);
    println!("Attempting to move {}", after_move);
    main_out.send(MessageToBot::Move(after_move.board)).unwrap();

    let bot_move = BoardPosition {
        board: main_in.recv().unwrap(),
        ..Default::default()
    };
    println!("{}", bot_move);
    println!("stopping bot");
    main_out.send(MessageToBot::Stop).unwrap();
    bot.join().unwrap();
}
