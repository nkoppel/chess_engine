use crate::movegen::*;

impl Moves {
    pub fn len(&self) -> usize {
        let mut out = self.others.len() as u32;

        for (_, bit) in self.bits.iter() {
            out += bit.count_ones();
        }

        out as usize
    }

    pub fn is_empty(&self) -> bool {
        self.others.len() == 0 && self.bits.len() == 0
    }

    pub fn clear(&mut self) {
        self.bits.clear();
        self.others.clear();
    }

    fn loc_to_string(mut loc: u8, inverted: bool) -> String {
        if inverted {
            Board::invert_loc(&mut loc);
        }

        String::from_utf8(vec![loc % 8 + 97, loc / 8 + 49]).unwrap()
    }

    pub fn move_to_string(m: &Move, inverted: bool) -> String {
        let mut out = String::new();

        let (from, to) =
            match m {
                Basic(f, t) => (*f, *t),
                En_passant(f, t) => (f + 32, t + 40),
                Castle_queen => (4, 2),
                Castle_king => (4, 6),
                Promotion(_, f, t) => (*f, *t)
            };

        out.push_str(&Moves::loc_to_string(from, inverted));
        out.push_str(&Moves::loc_to_string(to  , inverted));

        if let Promotion(piece, _, _) = m {
            out.push_str(
                match piece {
                    Queen => "q",
                    Knight => "n",
                    Rook => "r",
                    Bishop => "b",
                    _ => panic!("Invalid promotion!")
                }
            );
        }

        out
    }

    pub fn string_to_move(s: &str, board: &Board) -> Move {
        let mut i = s.chars();

        let x1 = i.next().unwrap().to_digit(18).unwrap() - 10;
        let y1 = i.next().unwrap().to_digit( 9).unwrap() - 1;
        let x2 = i.next().unwrap().to_digit(18).unwrap() - 10;
        let y2 = i.next().unwrap().to_digit( 9).unwrap() - 1;
        let mut out1 = (x1 + y1 * 8) as u8;
        let mut out2 = (x2 + y2 * 8) as u8;

        if board.inverted {
            Board::invert_loc(&mut out1);
            Board::invert_loc(&mut out2);
        }

        let loc1 = (x1 + y1 * 8) as isize;
        let loc2 = (x2 + y2 * 8) as isize;

        match board.get_loc_piece(loc1 as u8) {
            King => {
                if loc2 - loc1 == 2 {
                    return Castle_king;
                }
                if loc1 - loc2 == 2 {
                    return Castle_queen;
                }
            },
            Pawn => {
                if loc2 >= 56 {
                    let piece =
                        match i.next().unwrap() {
                            'q' => Queen ,
                            'n' => Knight,
                            'r' => Rook  ,
                            'b' => Bishop,
                            _ => panic!("Invalid promotion!")
                        };
                    return Promotion(piece, out1, out2);
                }
                if (loc2 - loc1) % 8 != 0 && board.get_loc_piece(loc2 as u8) == Empty {
                    return En_passant(out1 % 8, out2 % 8);
                }
            },
            _ => {}
        }

        Basic(out1, out2)
    }
}

impl Iterator for Moves {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.others.is_empty() {
            self.others.pop()
        } else if !self.bits.is_empty() {
            let mut bits = self.bits.last_mut().unwrap();
            while bits.1 == 0 {
                self.bits.pop();
                if self.bits.is_empty() {
                    return None;
                }
                bits = self.bits.last_mut().unwrap();
            }
            let to = bits.1.trailing_zeros();
            bits.1 ^= 1 << to;
            Some(Basic(bits.0, to as u8))
        } else {
            None
        }
    }
}
