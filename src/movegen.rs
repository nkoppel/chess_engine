#[derive(Clone, Debug, PartialEq)]
pub enum Move {
    Basic(u8, u8),
    // file of pawn which took, file of taken pawn
    En_passant(u8, u8),
    Castle_king,
    Castle_queen,
    Promotion(Piece, u8, u8),
}

pub use Move::*;
pub use crate::board::{Board, Piece, Piece::*};
use crate::gen_table::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Moves {
    pub bits: Vec<(u8, u64)>,
    pub others: Vec<Move>
}

impl Moves {
    pub fn new() -> Moves {
        Moves {
            bits: Vec::new(),
            others: Vec::new(),
        }
    }
}

fn nth_one(mut num: u64, n: usize) -> u8 {
    for i in 0..n {
        num &= num - 1;
    }

    num.trailing_zeros() as u8
}

impl Board {
    pub fn gen_moves_bits(&self, tables: &Tables) -> Vec<(u8, u64)> {
        let mut out = Vec::with_capacity(25);
        let all = self.all();

        for loc in LocStack(self.bishop & self.curr) {
            let mut att = all;
            let (mask, magic, offset) = tables.bishop[loc];
            att &= mask;
            att = att.overflowing_mul(magic).0;
            att >>= 55;
            att += offset;

            out.push((loc as u8, tables.magic[att as usize] & !self.curr));
        }

        for loc in LocStack(self.rook & self.curr) {
            let mut att = all;
            let (mask, magic, offset) = tables.rook[loc];
            att &= mask;
            att = att.overflowing_mul(magic).0;
            att >>= 52;
            att += offset;

            out.push((loc as u8, tables.magic[att as usize] & !self.curr));
        }

        for loc in LocStack(self.knight() & self.curr) {
            out.push((loc as u8, tables.knight[loc] & !self.curr));
        }

        for loc in LocStack(self.pawn & 0x0000ffffffffff00 & self.curr) {
            let mut moves = tables.pawn_moves[loc] & !all;

            if moves != 0 && loc / 8 == 1 {
                moves |= 1 << (loc + 16);
                moves &= !all;
            }

            moves |= tables.pawn_takes[loc] & self.other;

            if moves != 0 {
                out.push((loc as u8, moves));
            }
        }

        out.push((self.cking, tables.king[self.cking as usize] & !self.curr));

        out
    }

    pub fn threats(&self, tables: &Tables) -> u64 {
        let mut out = 0;
        let all = self.all() & !(1 << self.cking);

        for loc in LocStack(self.bishop & self.other) {
            let mut att = all;
            let (mask, magic, offset) = tables.bishop[loc];
            att &= mask;
            att = att.overflowing_mul(magic).0;
            att >>= 55;
            att += offset;

            out |= tables.magic[att as usize];
        }

        for loc in LocStack(self.rook & self.other) {
            let mut att = all;
            let (mask, magic, offset) = tables.rook[loc];
            att &= mask;
            att = att.overflowing_mul(magic).0;
            att >>= 52;
            att += offset;

            out |= tables.magic[att as usize];
        }

        for loc in LocStack(self.knight() & self.other) {
            out |= tables.knight[loc];
        }

        for loc in LocStack(self.pawns() & self.other) {
            out |= tables.other_pawn_takes[loc];
        }

        out |= tables.king[self.oking as usize];

        out
    }

    pub fn gen_moves_special(&self, tables: &Tables) -> (Vec<Move>, u64) {
        let mut out = Vec::new();

        if self.pawn >> 56 != 0 {
            let ep_file = 7 - self.pawn.leading_zeros();

            if ep_file != 0 && self.pawn & self.curr & (1 << (ep_file + 31)) != 0 {
                out.push(En_passant(ep_file as u8 - 1, ep_file as u8))
            }

            if ep_file != 7 && self.pawn & self.curr & (1 << (ep_file + 33)) != 0 {
                out.push(En_passant(ep_file as u8 + 1, ep_file as u8))
            }
        }

        let promote_pawns = self.pawn & self.curr & 0x00ff000000000000;
        let all = self.curr | self.other;

        for loc in LocStack(promote_pawns) {
            let mut moves = tables.pawn_moves[loc] & !all;

            moves |= tables.pawn_takes[loc] & self.other;

            for to in LocStack(moves) {
                let l = loc as u8;
                let t = to  as u8;
                out.push(Promotion(Queen , l, t));
                out.push(Promotion(Bishop, l, t));
                out.push(Promotion(Rook  , l, t));
                out.push(Promotion(Knight, l, t));
            }
        }

        let all = self.all();
        let mut threat = 0;

        if self.castle_curr[0] && all & 0b00001110 == 0 {
            threat = self.threats(tables);

            if threat & 0b00011000 == 0 {
                out.push(Castle_queen);
            }
        }

        if self.castle_curr[1] && all & 0b01100000 == 0 {
            if threat == 0 {
                threat = self.threats(tables);
            }

            if threat & 0b00110000 == 0 {
                out.push(Castle_king);
            }
        }

        (out, threat)
    }

    pub fn gen_moves(&self, tables: &Tables) -> (Moves, u64) {
        let (other, threats) = self.gen_moves_special(tables);
        (
            Moves {
                bits: self.gen_moves_bits(tables),
                others: other
            },
            threats
        )
    }

    pub fn do_move(&mut self, m: &Move) {
        match m {
            Basic(from, to) => {
                let piece = self.get_loc_piece(*from);
                self.clear_loc(*to);
                self.set_loc(*to, piece, false);
                self.clear_loc(*from);

                if *to == 0 && self.get_loc_piece(0) == Rook {
                    self.castle_other[0] = false;
                }
                if *to == 7 && self.get_loc_piece(7) == Rook {
                    self.castle_other[1] = false;
                }
                if piece == King {
                    self.castle_curr[0] = false;
                    self.castle_curr[1] = false;
                } else if piece == Rook {
                    if *from == 0 {
                        self.castle_curr[0] = false;
                    } else if *from == 7 {
                        self.castle_curr[1] = false;
                    }
                }
            },
            // file of pawn which took, file of taken pawn
            En_passant(fromfile, tofile) => {
                self.clear_loc(tofile + 32);
                self.clear_loc(fromfile + 32);
                self.set_loc(tofile + 40, Pawn, false);
            },
            Castle_king => {
                self.clear_loc(4);
                self.clear_loc(7);
                self.set_loc(6, King, false);
                self.set_loc(5, Rook, false);
                self.castle_curr[0] = false;
                self.castle_curr[1] = false;
            },
            Castle_queen => {
                self.clear_loc(4);
                self.clear_loc(0);
                self.set_loc(2, King, false);
                self.set_loc(3, Rook, false);
                self.castle_curr[0] = false;
                self.castle_curr[1] = false;
            },
            Promotion(p, from, to) => {
                self.set_loc(*to, *p, false);
                self.clear_loc(*from);
            },
        }
        self.pawn &= 0x00ffffffffffffff;
    }

    pub fn get_random_move(&self, moves: &Moves) -> Move {
        let mut ind = rand::random::<usize>() % moves.len();

        if ind < moves.others.len() {
            return moves.others[ind].clone();
        } else {
            ind -= moves.others.len();
            let mut i = 0;
            let mut ones = moves.bits[i].1.count_ones() as usize;

            while i < moves.bits.len() && ind >= ones {
                ind -= ones;
                i += 1;
                ones = moves.bits[i].1.count_ones() as usize;
            }

            let (from, bit) = moves.bits[i];

            Basic(from, nth_one(bit, ind))
        }
    }

    pub fn do_random_move(&mut self, moves: &Moves) -> Move {
        let mov = self.get_random_move(moves);
        self.do_move(&mov);
        mov
    }
}
