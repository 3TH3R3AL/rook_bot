use std::fmt;
#[allow(dead_code)]
#[derive(Clone, Copy)]
enum Color {
    White,
    Black,
}
#[allow(dead_code)]
#[derive(Clone, Copy)]
enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook { has_moved: bool },
    King { has_moved: bool },
    Queen,
    Empty,
}

use Color::*;
use PieceType::*;
#[derive(Clone, Copy)]
struct Piece {
    color: Color,
    piece_type: PieceType,
}
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
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.character())
    }
}

struct Board([[Piece; 8]; 8]);

impl Board {
    fn new(init_arr: [[(Color, PieceType); 8]; 8]) -> Board {
        Board(init_arr.map(|row| row.map(|cell| Piece::new(cell.0, cell.1))))
    }
    // fn new(init_arr: [[(Color, PieceType); 8]; 8]) -> Board {
    //     let mut new_board = Board([[Piece::new(Color::White, PieceType::Pawn); 8]; 8]);
    //     for i in 0..8 {
    //         for j in 0..8 {
    //             new_board.0[i][j] = Piece::new(init_arr[i][j].0, init_arr[i][j].1)
    //         }
    //     }
    //     new_board
    // }
}

// impl fmt::Display for Board {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//          #write!(f, "{}", self.character())
//     }
// }

//#[allow(nonstandard_style())]
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
    let init_board = Board::new(INITIAL_BOARD);
    println!("{}", init_board.0[0][0]);
}
