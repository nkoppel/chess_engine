use crate::board::{Board, Piece, Piece::*};
use crate::gen_table::{LocStack, Tables, new_tables, print_board};
use crate::movegen::{Move::*, Move, Moves};

const LIGHT_SQUARES: u64 = 0x55AA55AA55AA55AA;
const DARK_SQUARES : u64 = 0xAA55AA55AA55AA55;

#[derive(Clone)]
pub struct Position<'a> {
    pub board: Board,
    pub fifty: usize,
    pub full_moves: usize,
    pub tables: &'a Tables,
    pub threats: u64,
    pub moves: Moves
}

impl Position<'_> {
    pub fn new(tables: &Tables) -> Position {
        Position {
            board: Board::new(),
            fifty: 0,
            full_moves: 1,
            tables,
            threats: 0,
            moves: Moves::new()
        }
    }

    pub fn from_fen<'a>(tables: &'a Tables, fen: &str) -> Position<'a> {
        let mut words = fen.split(" ");
        let mut out = Position::new(tables);

        out.board = Board::from_fen(words.next().unwrap());

        let player = words.next().unwrap().chars().next().unwrap();

        for c in words.next().unwrap().chars() {
            match c {
                'K' => out.board.castle_curr [1] = true,
                'Q' => out.board.castle_curr [0] = true,
                'k' => out.board.castle_other[1] = true,
                'q' => out.board.castle_other[0] = true,
                _ => {},
            }
        }

        match words.next().unwrap().chars().next().unwrap() {
            '-' => {},
            ch => {
                if let Some(mut n) = ch.to_digit(18) {
                    if n >= 10 {
                        n -= 10;
                        out.board.pawn |= 1 << (n + 56);
                    }
                }
            }
        }

        out.fifty      = words.next().unwrap().to_string().parse().unwrap();
        out.full_moves = words.next().unwrap().to_string().parse().unwrap();

        if player == 'b' {
            out.board.invert();
        }

        out
    }

    pub fn gen_moves(&self) -> Vec<Move> {
        let mut board;
        let mut threat;
        let mut out = Vec::new();

        for m in self.moves.clone() {
            board = self.board.clone();
            board.do_move(&m);
            threat = board.threats(&self.tables);

            if threat & (1 << board.cking) == 0 {
                out.push(m);
            }
        }

        out
    }

    pub fn do_move(&mut self, m: &Move) {
        self.threats = 0;
        self.fifty += 1;

        match m {
            En_passant(_, _) | Promotion(_, _, _) => {
                self.fifty = 0;
            },
            Basic(from, to) => {
                if self.board.pawns() & (1u64 << from) != 0 {
                    self.fifty = 0;
                } else if self.board.other & (1u64 << to) != 0 {
                    self.fifty = 0;
                }
            },
            _ => {}
        }
        if self.board.inverted {
            self.full_moves += 1;
        }
        self.board.do_move(m);
    }

    pub fn set_threats(&mut self) {
        if self.threats == 0 {
            self.threats = self.board.threats(&self.tables);
        }
    }

    pub fn set_moves(&mut self) {
        let (moves, threat) = self.board.gen_moves(&self.tables);

        self.moves = moves;

        if threat != 0 {
            self.threats = threat;
        } else {
            self.set_threats();
        }

        if self.is_in_check() {
            let loc = self.board.cking as usize;
            let all = self.board.all();
            let mut block_squares = 0;

            let mut att = all;
            let (mask, magic, offset) = self.tables.bishop[loc];
            att &= mask;
            att = att.overflowing_mul(magic).0;
            att >>= 55;
            att += offset;
            att = self.tables.magic[att as usize];

            if att & self.board.other & self.board.bishop != 0 {
                block_squares |= att & (self.threats | self.board.other);
            }

            att = all;
            let (mask, magic, offset) = self.tables.rook[loc];
            att &= mask;
            att = att.overflowing_mul(magic).0;
            att >>= 52;
            att += offset;
            att = self.tables.magic[att as usize];

            if att & self.board.other & self.board.rook != 0 {
                block_squares |= att & (self.threats | self.board.other);
            }

            block_squares |= self.tables.knight[loc] & self.board.other & self.board.knight();

            block_squares |= self.tables.other_pawn_takes[loc] & self.board.other & self.board.pawns();

            let king_moves = self.moves.bits.last().unwrap().clone();
            for m in self.moves.bits.iter_mut() {
                m.1 &= block_squares;
            }
            *self.moves.bits.last_mut().unwrap() = king_moves;
        }
    }

    pub fn do_random_move(&mut self) -> Move {
        let mov = self.board.get_random_move(&self.moves);
        self.do_move(&mov);
        mov
    }

    pub fn is_in_check(&self) -> bool {
        self.threats & (1 << self.board.cking) != 0
    }

    pub fn test_endgame(&mut self) -> Option<usize> {
        self.set_threats();

        if self.fifty >= 50 {
            return Some(1)
        }

        if self.board.pawn == 0 && self.board.rook == 0 {
            let num_knights = self.board.knight().count_ones();
            if self.board.bishop == 0 && num_knights <= 1 {
                return Some(1)
            }
            if num_knights == 0 &&
               (self.board.bishop & LIGHT_SQUARES == 0 ||
                self.board.bishop & DARK_SQUARES  == 0)
            {
                return Some(1)
            }
        }

        if self.tables.king[self.board.cking as usize] &
            !(self.board.curr | self.threats) != 0
        {
            return None;
        }

        let mut threat;

        let mut board = self.board.clone();

        for m in self.moves.clone() {
            board.do_move(&m);
            threat = board.threats(&self.tables);

            if threat & (1 << board.cking) == 0 {
                return None;
            }
            board = self.board.clone();
        }

        if self.is_in_check() {
            return Some(board.inverted as usize * 2);
        } else {
            return Some(1);
        }
    }

    pub fn do_rollout(&mut self) -> usize {
        loop {
            self.threats = 0;
            self.set_moves();
            self.set_threats();

            match self.test_endgame() {
                None => {},
                Some(x) => return x,
            }

            let mut board = self.board.clone();
            let mut mov = self.do_random_move();

            self.threats = 0;
            self.set_threats();

            while self.is_in_check() {
                self.board = board.clone();
                mov = self.do_random_move();
                if self.fifty != 0 {
                    self.fifty -= 1;
                }
                self.threats = 0;
                self.set_threats();
            }
            self.board.invert();
        }
    }
}
