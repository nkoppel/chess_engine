use crate::gen_table::print_board;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Piece {
    Empty,
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

use Piece::*;

#[derive(Eq, Hash, Clone, Debug, PartialEq)]
pub struct Board {
    pub pawn: u64,
    pub rook: u64,
    pub bishop: u64,
    pub curr: u64,
    pub other: u64,
    pub cking: u8,
    pub oking: u8,
    pub inverted: bool,
    pub castle_curr: [bool; 2],
    pub castle_other: [bool; 2],
}

impl Board {
    pub fn new() -> Board {
        Board {
            pawn: 0,
            rook: 0,
            bishop: 0,
            curr: 0,
            other: 0,
            cking: 0,
            oking: 0,
            inverted: false,
            castle_curr: [false; 2],
            castle_other: [false; 2],
        }
    }

    pub fn from_fen(fen: &str) -> Board {
        let mut x = 0;
        let mut y = 7;
        let mut out = Board::new();
        for c in fen.chars() {
            let bit = 1 << (x + y * 8);
            let mut color =
                if c.is_ascii_lowercase() {
                    &mut out.other
                } else {
                    &mut out.curr
                };

            match c.to_ascii_lowercase() {
                'p' => {
                    *color |= bit;
                    out.pawn |= bit;
                    x += 1;
                },
                'n' => {
                    *color |= bit;
                    x += 1;
                },
                'b' => {
                    *color |= bit;
                    out.bishop |= bit;
                    x += 1;
                },
                'r' => {
                    *color |= bit;
                    out.rook |= bit;
                    x += 1;
                },
                'q' => {
                    *color |= bit;
                    out.bishop |= bit;
                    out.rook |= bit;
                    x += 1;
                },
                'k' => {
                    *color |= bit;
                    if c.is_ascii_lowercase() {
                        out.oking = x + y * 8;
                    } else {
                        out.cking = x + y * 8;
                    }
                    x += 1;
                },
                _ => {
                    if c.is_ascii_digit() {
                        x += c.to_digit(10).unwrap() as u8;
                    }
                }
            }
            if x >= 8 && y > 0 {
                x -= 8;
                y -= 1;
            }
        }

        out
    }

    pub fn pawns(&self) -> u64 {
        // filter out en-passant codes on ranks 1 and 8
        0x00ffffffffffff00 & self.pawn
    }

    pub fn knight(&self) -> u64 {
        let all = (self.curr | self.other);

        // get all knights and kings
        let knk = all & !(self.pawns() | self.rook | self.bishop);

        // filter out kings
        knk & !(1 << self.cking | 1 << self.oking)
    }

    pub fn queen(&self) -> u64 {
        self.rook & self.bishop
    }

    pub fn curr_queen(&self) -> u64 {
        self.curr & self.rook & self.bishop
    }

    pub fn all(&self) -> u64 {
        self.curr | self.other
    }

    pub fn invert_loc(l: &mut u8) {
        *l = (*l % 8) + 8 * (7 - *l / 8);
    } 

    pub fn invert(&mut self) {
        self.pawn = self.pawn.swap_bytes();
        self.rook = self.rook.swap_bytes();
        self.bishop = self.bishop.swap_bytes();
        self.curr = self.curr.swap_bytes();
        self.other = self.other.swap_bytes();
        self.inverted = !self.inverted;
        Board::invert_loc(&mut self.cking);
        Board::invert_loc(&mut self.oking);
        std::mem::swap(&mut self.curr, &mut self.other);
        std::mem::swap(&mut self.cking, &mut self.oking);
        std::mem::swap(&mut self.castle_curr, &mut self.castle_other);
    }

    pub fn get_loc_piece(&self, loc: u8) -> Piece {
        let bit = 1 << loc;

        if self.curr & bit == 0 && self.other & bit == 0 {
            Empty
        } else if self.bishop & bit != 0 {
            if self.rook & bit != 0 {
                Queen
            } else {
                Bishop
            }
        } else if self.rook & bit != 0 {
            Rook
        } else if self.pawns() & bit != 0 {
            Pawn
        } else if loc == self.cking || loc == self.oking {
            King
        } else {
            Knight
        }
    }

    pub fn get_loc(&self, loc: u8) -> (Piece, bool) {
        let bit = 1 << loc;

        match self.get_loc_piece(loc) {
            Empty => (Empty, false),
            piece => {
                (piece, self.other & bit != 0)
            }
        }
    }

    pub fn clear_loc(&mut self, loc: u8) {
        let bit = !(1 << loc);

        if loc > 7 && loc < 56 {
            self.pawn &= bit;
        }
        self.bishop &= bit;
        self.rook &= bit;
        self.curr &= bit;
        self.other &= bit;
    }

    pub fn set_loc(&mut self, loc: u8, piece: Piece, is_other: bool) {
        let bit = 1 << loc;

        let mut color =
            if is_other {
                &mut self.other
            } else {
                &mut self.curr
            };

        match piece {
            Pawn => {
                *color |= bit;
                self.pawn |= bit;
            },
            Knight => {
                *color |= bit;
            },
            Bishop => {
                *color |= bit;
                self.bishop |= bit;
            },
            Rook => {
                *color |= bit;
                self.rook |= bit;
            },
            Queen => {
                *color |= bit;
                self.bishop |= bit;
                self.rook |= bit;
            },
            King => {
                *color |= bit;
                if is_other {
                    self.oking = loc;
                } else {
                    self.cking = loc;
                }
            },
            Empty => {}
        }
    }

    fn copy_bit(num: &mut u64, b1: u64, b2: u64) {
        if *num & b1 != 0 {
            *num |= b2;
        } else {
            *num &= !b2;
        }

        *num ^= b1;
    }

    pub fn copy_loc(&mut self, l1: u8, l2: u8) {
        let bit1 = 1 << l1;
        let bit2 = 1 << l2;
        Board::copy_bit(&mut self.curr  , bit1, bit2);
        Board::copy_bit(&mut self.other , bit1, bit2);
        Board::copy_bit(&mut self.bishop, bit1, bit2);
        Board::copy_bit(&mut self.rook  , bit1, bit2);
        if l1 < 56 {
            Board::copy_bit(&mut self.pawn, bit1, bit2);
        }
        if l1 == self.cking {
            self.cking = l2;
        }
    }

    pub fn debug_print(&self) {
        // let mut tmp;
        let pboard = self;
            // if self.inverted {
                // tmp = self.clone();
                // tmp.invert();
                // &tmp
            // } else {
                // self
            // };

        println!("+---+---+---+---+---+---+---+---+");
        for y in (0..8).rev() {
            print!("|");

            for x in (0..8) {
                print!("{:#3 }|", x + y * 8);
            }

            println!("");

            print!("|");
            for x in (0..8) {
                let bit = 1 << (x + y * 8);

                if pboard.curr & bit != 0 {
                    print!("C");
                } else {
                    print!(" ");
                }

                if pboard.rook & bit != 0 {
                    print!("R");
                } else {
                    print!(" ");
                }

                if pboard.bishop & bit != 0 {
                    print!("B");
                } else {
                    print!(" ");
                }
                print!("|");
            }

            println!("");

            print!("|");
            for x in (0..8) {
                let loc = x + y * 8;
                let bit = 1 << loc;

                if pboard.other & bit != 0 {
                    print!("O");
                } else {
                    print!(" ");
                }

                if pboard.pawn & bit != 0 {
                    print!("P");
                } else {
                    print!(" ");
                }

                if pboard.cking == loc {
                    print!("C");
                } else if pboard.oking == loc {
                    print!("O");
                } else {
                    print!(" ");
                }
                print!("|");
            }
            println!("");
            println!("+---+---+---+---+---+---+---+---+");
        }
    }
} 

use std::fmt;

fn piece_to_string(piece: Piece) -> String {
    match piece {
        Pawn   => "♟",
        Knight => "♞",
        Bishop => "♝",
        Rook   => "♜",
        Queen  => "♛",
        King   => "♚",
        Empty  => " "
    }.to_string()
}

use colored::*;

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tmp;
        let pboard =
            if self.inverted {
                tmp = self.clone();
                tmp.invert();
                &tmp
            } else {
                self
            };

        for y in (0..8).rev() {
            write!(f, "{}  ", y + 1)?;
            if y % 2 == 0 {
                write!(f, "{}", "▐".red())?;
            } else {
                write!(f, "{}", "▐".yellow())?;
            }

            for x in 0..8 {
                let pos = (x + y * 8);
                let bit = 1 << pos;

                let mut s = piece_to_string(pboard.get_loc_piece(pos));

                // s.insert(0, ' ');
                // s.push(' ');

                let mut s =
                    if bit & pboard.curr != 0 {
                        s.white()
                    } else {
                        s.black()
                    };

                let mut s2;

                if (x + y) % 2 == 0 {
                    s = s.on_red();
                    s2 = "▐".yellow().on_red();
                } else {
                    s = s.on_yellow();
                    s2 = "▐".red().on_yellow();
                }

                if x == 7 {
                    s2 = s2.black();
                }

                write!(f, "{}{}", s, s2);
            }

            writeln!(f, "")?;
        }

        writeln!(f, "")?;
        writeln!(f, "    A B C D E F G H")?;
        Ok(())
    }
}

