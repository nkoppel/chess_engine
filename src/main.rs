#![feature(test)]
extern crate test;

pub const TEST_BOARD: &str = "1kr4r/1bq1pp1p/pn3Pp1/1pp4n/4P2P/P1NNQP1B/1PP5/2KR3R";

mod gen_table;
mod board;
mod movegen;
mod moves;
mod position;
mod search;
mod eval;

use crate::gen_table::*;
use crate::board::{Board, Piece, Piece::*};
use crate::movegen::{*, Move::*};
use crate::position::Position;
use crate::search::*;

use std::io::Write;

fn main() {
    let tables = new_tables();
    // let mut position = Position::from_fen(&tables, "1kr4r/1bq1pp1p/pn3Pp1/1pp4n/4P2P/P1NNQP1B/1PP5/2KR3R w - - 0 1");
    let mut position = Position::from_fen(&tables, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    let mut tree = GameTree::new();

    let mut pos = position.clone();
    // position.board.invert();
    // println!("{}", position.board);
    let stdin = std::io::stdin();
    println!("{}", position.board);
    position.set_moves();
    while position.test_endgame() == None {
        // for i in 0..10000 {
            // tree.search(position.clone());
        // }
        // let mov = tree.get_best_move();
        // for m in tree.get_searched_line() {
            // print!("{} ", Moves::move_to_string(&m, position.board.inverted));
        // }
        // println!();

        // tree.do_move(&mov);
        // println!("{}", Moves::move_to_string(&mov, position.board.inverted));
        // position.do_move(&mov);
        // position.board.invert();
        // println!("{}", position.board);
        // position.set_moves();

        // if position.test_endgame() != None {
            // break;
        // }
        // let mut buf = String::new();
        // print!("Enter your move: ");
        // std::io::stdout().flush();
        // stdin.read_line(&mut buf);
        // let mut mov = Moves::string_to_move(&buf, &position.board);
        // position.set_moves();
        // let moves = position.gen_moves();
        // while !moves.contains(&mov) {
            // println!("Invalid move!");
            // print!("Enter your move: ");
            // std::io::stdout().flush();
            // buf.clear();
            // stdin.read_line(&mut buf);
            // mov = Moves::string_to_move(&buf, &position.board);
        // }
        // position.do_move(&mov);
        // position.board.invert();
        // println!("{}", position.board);
        // position.set_moves();

        if position.test_endgame() != None {
            break;
        }

        let mov = ab_search(position.clone(), 3000).unwrap();

        tree.do_move(&mov);
        println!("{}", Moves::move_to_string(&mov, position.board.inverted));
        position.do_move(&mov);
        position.board.invert();
        println!("{}", position.board);
        position.set_moves();
    }
}

mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn test_ray_mask_1() {
        let mut dummy = 0;
        ray_mask(1, (-1, 0), &mut dummy);
        assert_eq!(dummy, 0);
    }

    #[test]
    fn test_ray_mask_2() {
        let mut dummy = 0;
        ray_mask(2, (-1, 0), &mut dummy);
        assert_eq!(dummy, 2);
    }

    #[test]
    fn test_new_tables() {
        new_tables();
    }

    #[test]
    fn test_board_invert() {
        let mut board1 = Board::from_fen(TEST_BOARD);
        let mut board2 = Board::from_fen("2kr3r/1pp5/p1nnqp1b/4p2p/1PP4N/PN3pP1/1BQ1PP1P/1KR4R");
        board1.invert();
        board2.inverted = true;
        assert_eq!(board1, board2);
        board1.invert();
        board1.invert();
        assert_eq!(board1, board2);
    }

    #[test]
    fn test_gen_moves() {
        let board = Board::from_fen(TEST_BOARD);
        let tables = new_tables();

        let (moves, threats) = board.gen_moves(&tables);

        assert_eq!(moves.len(), 45);
        assert_eq!(threats, 0);
    }

    #[test]
    fn test_do_move() {
        let mut board;

        // Test Basic 1
        board = Board::from_fen("8/8/8/8/8/8/8/R7");
        board.do_move(&Basic(0, 7));
        assert_eq!(board, Board::from_fen("8/8/8/8/8/8/8/7R"));

        // Test Basic 2
        board = Board::from_fen("8/8/8/8/8/8/8/R6b");
        board.do_move(&Basic(0, 7));
        assert_eq!(board, Board::from_fen("8/8/8/8/8/8/8/7R"));

        // Test En_passant
        board = Board::from_fen("8/8/8/3Pp3/8/8/8/8");
        board.pawn |= 1 << 60;
        board.do_move(&En_passant(3, 4));
        assert_eq!(board, Board::from_fen("8/8/4P3/8/8/8/8/8"));

        // Test Castle_king
        board = Board::from_fen("8/8/8/8/8/8/8/4K2R");
        board.do_move(&Castle_king);
        assert_eq!(board, Board::from_fen("8/8/8/8/8/8/8/5RK1"));

        // Test Castle_queen
        board = Board::from_fen("8/8/8/8/8/8/8/R3K3");
        board.do_move(&Castle_queen);
        assert_eq!(board, Board::from_fen("8/8/8/8/8/8/8/2KR4"));

        // Test Promotion
        board = Board::from_fen("8/P/8/8/8/8/8/8");
        board.do_move(&Promotion(Queen, 48, 56));
        assert_eq!(board, Board::from_fen("Q7/8/8/8/8/8/8/8"));

        // Test Promotion_take
        board = Board::from_fen("1b6/P/8/8/8/8/8/8");
        board.do_move(&Promotion(Queen, 48, 57));
        assert_eq!(board, Board::from_fen("1Q6/8/8/8/8/8/8/8"));
    }

    #[test]
    fn test_test_endgame() {
        let tables = new_tables();
        let mut position = Position::from_fen(&tables, "Kqk5/8/8/8/8/8/8/8 w - - 0 1");
        assert_eq!(position.test_endgame(), Some(2));
    }

    #[bench]
    fn bench_board_invert(b: &mut Bencher) {
        let mut board = Board::from_fen(TEST_BOARD);

        b.iter(|| test::black_box(&mut board).invert());
    }

    #[bench]
    fn bench_board_clone(b: &mut Bencher) {
        let mut board = Board::from_fen(TEST_BOARD);

        b.iter(|| test::black_box(&board).clone());
    }

    #[bench]
    fn bench_gen_moves_bits(b: &mut Bencher) {
        let mut board = Board::from_fen(TEST_BOARD);
        let tables = new_tables();

        b.iter(|| test::black_box(&board).gen_moves_bits(&tables));
    }

    #[bench]
    fn bench_gen_moves_special(b: &mut Bencher) {
        let mut board = Board::from_fen(TEST_BOARD);
        let tables = new_tables();

        b.iter(|| test::black_box(&board).gen_moves_special(&tables));
    }

    #[bench]
    fn bench_gen_moves(b: &mut Bencher) {
        let mut board = Board::from_fen(TEST_BOARD);
        let tables = new_tables();

        b.iter(|| test::black_box(&board).gen_moves(&tables));
    }

    #[bench]
    fn bench_get_loc(b: &mut Bencher) {
        let mut board = Board::from_fen(TEST_BOARD);
        
        b.iter(|| test::black_box(&board).get_loc(18));
    }

    #[bench]
    fn bench_clear_loc(b: &mut Bencher) {
        let mut board = Board::from_fen(TEST_BOARD);
        
        b.iter(|| test::black_box(&mut board).clear_loc(18));
    }

    #[bench]
    fn bench_set_loc(b: &mut Bencher) {
        let mut board = Board::from_fen(TEST_BOARD);
        
        b.iter(|| test::black_box(&mut board).set_loc(0, Queen, true));
    }

    #[bench]
    fn bench_copy_loc(b: &mut Bencher) {
        let mut board = Board::from_fen("Q7/8/8/8/8/8/8/8");
        
        b.iter(|| test::black_box(&mut board).copy_loc(0, 1));
    }

    #[bench]
    fn bench_threats(b: &mut Bencher) {
        let mut board = Board::from_fen(TEST_BOARD);
        let tables = new_tables();
        
        b.iter(|| test::black_box(&board).threats(&tables));
    }

    #[bench]
    fn bench_do_random_move(b: &mut Bencher) {
        let mut board = Board::from_fen(TEST_BOARD);
        let tables = new_tables();
        let moves = board.gen_moves(&tables).0;
        
        b.iter(|| {
            let mut board2 = board.clone();
            board2.do_random_move(&moves)
        });
    }

    #[bench]
    fn bench_test_endgame(b: &mut Bencher) {
        let tables = new_tables();
        let mut pos = Position::from_fen(&tables, "k7/2Q5/4R3/5R2/8/8/r7/rK6 w - - 0 1");
        pos.set_moves();

        b.iter(|| {
            test::black_box(&mut pos.clone()).test_endgame()
        });
    }

    #[bench]
    fn bench_do_rollout(b: &mut Bencher) {
        let tables = new_tables();
        let pos = Position::from_fen(&tables, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        b.iter(|| {
            test::black_box(&mut pos.clone()).do_rollout()
        });
    }

    #[bench]
    fn bench_moves_len(b: &mut Bencher) {
        let mut board = Board::from_fen(TEST_BOARD);
        let tables = new_tables();
        let moves = board.gen_moves(&tables).0;
        
        b.iter(|| moves.len());
    }

    #[bench]
    fn bench_do_move_basic(b: &mut Bencher) {
        let mut board = Board::from_fen(TEST_BOARD);
        let tables = new_tables();
        
        b.iter(|| {
            let mut board2 = board.clone();
            test::black_box(&mut board2).do_move(&Basic(23, 58))
        });
    }

    #[bench]
    fn bench_tree_traversal(b: &mut Bencher) {
        let tables = new_tables();
        let mut position = Position::from_fen(&tables, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        let mut tree = GameTree::new();
        for i in 0..5000 {
            tree.search(position.clone());
            println!("{}", i);
        }

        b.iter(|| test::black_box(&mut tree).search(position.clone()));
    }

    #[bench]
    fn bench_eval(b: &mut Bencher) {
        let tables = new_tables();
        let mut position = Position::from_fen(&tables, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        b.iter(|| eval::eval(test::black_box(&mut position)));
    }

    #[bench]
    fn bench_alphabeta(b: &mut Bencher) {
        let tables = new_tables();
        let mut position = Position::from_fen(&tables, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        b.iter(|| best_move(test::black_box(position.clone()), 1, None));
    }
}
