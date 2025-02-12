#![allow(unused)]
// I'll use these as index alias for my array of bitboards
const WHITE_PAWN:   usize = 0;
const WHITE_KNIGHT: usize = 1;
const WHITE_BISHOP: usize = 2;
const WHITE_ROOK:   usize = 3;
const WHITE_QUEEN:  usize = 4;
const WHITE_KING:   usize = 5;
const BLACK_PAWN:   usize = 6;
const BLACK_KNIGHT: usize = 7;
const BLACK_BISHOP: usize = 8;
const BLACK_ROOK:   usize = 9;
const BLACK_QUEEN:  usize = 10;
const BLACK_KING:   usize = 11;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    White,
    Black,
}

type BitBoard = u64;
type BitBoards = [BitBoard; 12];

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Coords {
    x: u8,
    y: u8,
}

impl Coords {
    pub fn from_str(coord: &str) -> Result<Coords, String> {
        if coord.len() != 2 {
            return Err("Invalid coordinate format".to_string());
        }

        let file = coord.chars().nth(0).unwrap();
        let rank = coord.chars().nth(1).unwrap();

        if !('a'..='h').contains(&file) || !('1'..='8').contains(&rank) {
            return Err("Coordinate out of bounds".to_string());
        }

        let x = (file as u8) - b'a';
        let y = (rank as u8) - b'1';

        Ok(Coords { x, y })
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct Board {
    // Piece placement data
    bit_boards: BitBoards,

    // Current player
    active_color: Color,

    // Castling availability
    white_oo: bool,
    white_ooo: bool,
    black_oo: bool,
    black_ooo: bool,

    // En passant target square. The square that can be captured.
    en_passant: Option<Coords>,
    // The number of halfmoves since the last capture or pawn advance. fifty-move rule
    half_moves: u16,
    // The number of the full moves. It starts at 1 and is incremented after Black's move.
    full_moves: u16,
}

impl Board {
    pub fn empty_board() -> Board {
        // I set the struct param values to what I expect would be default initialization values
        Board {
            bit_boards: [0; 12],
            active_color: Color::White,
            white_oo: false,
            white_ooo: false,
            black_oo: false,
            black_ooo: false,
            en_passant: None,
            half_moves: 0,
            full_moves: 0,
        }
    }

    pub fn from_fen(fen: &str) -> Result<Board, String> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() != 6 {
            return Err("Invalid FEN string".to_string());
        }

        let piece_placement = parts[0];
        let active_color = parts[1];
        let castling_availability = parts[2];
        let en_passant_target = parts[3];
        let halfmove_clock = parts[4];
        let fullmove_number = parts[5];

        let mut board = Board::empty_board();
        let mut x = 0;
        let mut y = 7;

        for c in piece_placement.chars() {
            if c == '/' {
                x = 0;
                if y > 0 {
                    y -= 1;
                }
                continue;
            }
            if let Some(empty_squares) = c.to_digit(10) {
                x += empty_squares as u8;
            } else if let Some(piece_type) = Board::piece_from_char(c) {
                if x < 8 && y < 8 {
                    let pos = Coords { x, y };
                    board.place_piece(piece_type, pos);
                    x += 1;
                }
            } else {
                return Err("Invalid character in FEN piece placement".to_string());
            }
        }

        board.active_color = match active_color {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err("Invalid FEN active color".to_string()),
        };

        board.white_oo = castling_availability.contains('K');
        board.white_ooo = castling_availability.contains('Q');
        board.black_oo = castling_availability.contains('k');
        board.black_ooo = castling_availability.contains('q');

        board.en_passant = if en_passant_target != "-" {
            Some(Coords::from_str(en_passant_target)?)
        } else {
            None
        };

        board.half_moves = halfmove_clock
            .parse::<u16>()
            .map_err(|_| "Invalid FEN halfmove clock".to_string())?;
        board.full_moves = fullmove_number
            .parse::<u16>()
            .map_err(|_| "Invalid FEN fullmove number".to_string())?;

        Ok(board)
    }

    pub fn startpos() -> Board {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    pub fn place_piece(&mut self, piece_type: usize, position: Coords) {
        if position.x > 7 || position.y > 7 {
            panic!(
                "Coordinates out of bounds: x = {}, y = {}",
                position.x, position.y
            );
        }
        let mask = 1u64 << (position.y * 8 + position.x);
        self.bit_boards[piece_type] |= mask;
    }

    pub fn remove_piece(&mut self, piece_type: usize, position: Coords) {
        if position.x > 7 || position.y > 7 {
            panic!(
                "Coordinates out of bounds: x = {}, y = {}",
                position.x, position.y
            );
        }
        let mask = !(1u64 << (position.y * 8 + position.x));
        self.bit_boards[piece_type] &= mask;
    }

    pub fn piece_from_char(c: char) -> Option<usize> {
        match c {
            'P' => Some(WHITE_PAWN),
            'N' => Some(WHITE_KNIGHT),
            'B' => Some(WHITE_BISHOP),
            'R' => Some(WHITE_ROOK),
            'Q' => Some(WHITE_QUEEN),
            'K' => Some(WHITE_KING),
            'p' => Some(BLACK_PAWN),
            'n' => Some(BLACK_KNIGHT),
            'b' => Some(BLACK_BISHOP),
            'r' => Some(BLACK_ROOK),
            'q' => Some(BLACK_QUEEN),
            'k' => Some(BLACK_KING),
            _ => None,
        }
    }

    pub fn print_board(&self) {
        let piece_symbols = ["P", "N", "B", "R", "Q", "K", "p", "n", "b", "r", "q", "k"];

        let mut display_board = [["."; 8]; 8];

        for (piece_type, &bitboard) in self.bit_boards.iter().enumerate() {
            for position in 0..64 {
                if (bitboard & (1u64 << position)) != 0 {
                    let x = position % 8;
                    let y = position / 8;
                    display_board[y][x] = piece_symbols[piece_type];
                }
            }
        }

        println!("Board:");
        for row in display_board.iter().rev() {
            for cell in row.iter() {
                print!("{} ", cell);
            }
            println!();
        }

        println!("Active Color: {:?}", self.active_color);
        println!(
            "Castling availability: {}{}{}{}",
            if self.white_oo { 'K' } else { '-' },
            if self.white_ooo { 'Q' } else { '-' },
            if self.black_oo { 'k' } else { '-' },
            if self.black_ooo { 'q' } else { '-' }
        );
        println!(
            "En passant: {}",
            self.en_passant.map_or("None".to_string(), |c| format!(
                "{}{}",
                (c.x + b'a') as char,
                c.y + 1
            ))
        );
        println!("Half moves: {}", self.half_moves);
        println!("Full moves: {}", self.full_moves);
    }
}

#[allow(unused_variables)]
fn main() {
    // empty board:
    let mut board = Board::from_fen("8/8/8/8/8/8/8/8 w - - 0 1").unwrap();
    // place a black rook:
    board.place_piece(BLACK_ROOK, Coords { x: 7, y: 7 });
    // print the board:
    board.print_board();
    println!("{:b}", board.bit_boards[BLACK_ROOK]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_fen_valid() {
        let board =
            Board::from_fen("rnbqkb1r/pp2pppp/2p2n2/3p4/3P4/2P2N2/PP2PPPP/RNBQKB1R w KQkq - 0 5")
                .unwrap();
        assert_eq!(board.active_color, Color::White);
        assert!(board.white_oo);
        assert!(board.white_ooo);
        assert!(board.black_oo);
        assert!(board.black_ooo);
        assert_eq!(board.en_passant, None);
        assert_eq!(board.half_moves, 0);
        assert_eq!(board.full_moves, 5);
    }

    #[test]
    fn test_from_fen_invalid() {
        assert!(Board::from_fen("invalid_fen").is_err());
    }

    #[test]
    fn test_place_piece() {
        let mut board = Board::empty_board();
        board.place_piece(WHITE_PAWN, Coords { x: 0, y: 1 });
        assert_eq!(board.bit_boards[WHITE_PAWN], 1u64 << 8);
    }

    #[test]
    fn test_piece_from_char() {
        assert_eq!(Board::piece_from_char('P'), Some(WHITE_PAWN));
        assert_eq!(Board::piece_from_char('k'), Some(BLACK_KING));
        assert_eq!(Board::piece_from_char('Z'), None);
    }

    #[test]
    fn test_castling_ability_parsing() {
        let board = Board::from_fen("8/8/8/8/8/8/8/8 w KQkq - 0 1").unwrap();
        assert!(board.white_oo);
        assert!(board.white_ooo);
        assert!(board.black_oo);
        assert!(board.black_ooo);

        let board2 = Board::from_fen("8/8/8/8/8/8/8/8 w - - 0 1").unwrap();
        assert!(!board2.white_oo);
        assert!(!board2.white_ooo);
        assert!(!board2.black_oo);
        assert!(!board2.black_ooo);
    }

    #[test]
    fn test_en_passant_square_parsing() {
        let board = Board::from_fen("8/8/8/8/4pP2/8/8/8 b - f3 0 1").unwrap();
        assert_eq!(board.en_passant, Some(Coords { x: 5, y: 2 }));

        let board2 = Board::from_fen("8/8/8/8/8/8/8/8 w KQkq - 0 1").unwrap();
        assert_eq!(board2.en_passant, None);
    }

    #[test]
    fn test_half_and_full_moves() {
        let board1 = Board::from_fen("8/8/8/8/8/8/8/8 w KQkq - 5 10").unwrap();
        assert_eq!(board1.half_moves, 5);
        assert_eq!(board1.full_moves, 10);

        let board2 =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(board2.half_moves, 0);
        assert_eq!(board2.full_moves, 1);
    }

    #[test]
    fn test_active_color_parsing() {
        let board_white = Board::from_fen("8/8/8/8/8/8/8/8 w - - 0 1").unwrap();
        assert_eq!(board_white.active_color, Color::White);

        let board_black = Board::from_fen("8/8/8/8/8/8/8/8 b - - 0 1").unwrap();
        assert_eq!(board_black.active_color, Color::Black);
    }
    #[test]
    fn test_full_board_fen_parsing() {
        let board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let white_pawn_board: BitBoard =
            0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000;
        let black_pawn_board: BitBoard =
            0b00000000_11111111_00000000_00000000_00000000_00000000_00000000_00000000;
        assert_eq!(board.bit_boards[WHITE_PAWN], white_pawn_board);
        assert_eq!(board.bit_boards[BLACK_PAWN], black_pawn_board);
        // Add further piece asserts as needed
    }

    #[test]
    fn test_empty_board() {
        let board = Board::empty_board();
        for piece_type in board.bit_boards.iter() {
            assert_eq!(*piece_type, 0);
        }
        assert_eq!(board.active_color, Color::White);
        assert!(!board.white_oo);
        assert!(!board.white_ooo);
        assert!(!board.black_oo);
        assert!(!board.black_ooo);
        assert_eq!(board.en_passant, None);
        assert_eq!(board.half_moves, 0);
        assert_eq!(board.full_moves, 0);
    }

    #[test]
    fn test_coords_from_str_valid() {
        let coords = Coords::from_str("a1").unwrap();
        assert_eq!(coords, Coords { x: 0, y: 0 });

        let coords = Coords::from_str("h8").unwrap();
        assert_eq!(coords, Coords { x: 7, y: 7 });
    }

    #[test]
    fn test_coords_from_str_invalid() {
        assert!(Coords::from_str("i1").is_err());
        assert!(Coords::from_str("a9").is_err());
        assert!(Coords::from_str("ab").is_err());
    }
}
