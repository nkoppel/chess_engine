use crate::position::*;
use crate::movegen::{Moves, Move, Move::*};
use crate::board::Board;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct GameTree {
    pub ucb: f64,
    pub score: f64,
    pub visits: usize,
    pub black: bool,
    pub endgame: bool,
    pub mov: Move,
    pub children: Vec<Rc<RefCell<GameTree>>>,
}

impl GameTree {
    pub fn new() -> GameTree {
        GameTree{
            ucb: 0.,
            score: 0.,
            visits: 0,
            black: false,
            endgame: false,
            mov: Basic(0,0),
            children: Vec::new(),
        }
    }

    fn get_searched_move_loc(&self) -> usize {
        let mut maxloc = 0;
        let mut max = 0.0;

        for i in 0..self.children.len() {
            let child = self.children[i].borrow();

            if child.ucb > max {
                max = child.ucb;
                maxloc = i;
            }
        }

        maxloc
    }

    fn calc_trust(&self) -> f64 {
        (self.visits as f64 / 2.5 - 15.).atan() / std::f64::consts::PI + 0.5
    }

    fn update_ucb(&mut self, parent_visits: usize) {
        self.ucb =
            self.score +
            std::f64::consts::SQRT_2 *
            ((parent_visits as f64).ln() / self.visits as f64)
    }

    fn expand(&mut self, position: &mut Position) {
        position.set_moves();
        self.children = position.gen_moves().into_iter().map(|m|
        {
            let mut pos = position.clone();
            pos.do_move(&m);
            pos.board.invert();
            let mut child = GameTree::new();
            child.black = position.board.inverted;
            child.visits = 1;
            child.mov = m;

            pos.set_moves();
            if let Some(n) = pos.test_endgame() {
                child.endgame = true;
                child.score = n as f64 / 2.;
            } else {
                child.score = pos.do_rollout() as f64 / 2.;
            }

            if !pos.board.inverted {
                child.score = 1. - child.score;
            }
            Rc::new(RefCell::new(child))
        }).collect();
    }

    pub fn search(&mut self, mut position: Position) {
        if self.endgame {
            self.visits += 1;
            return;
        }

        if self.children.is_empty() {
            self.expand(&mut position);
        } else {
            let mut child = self.children[self.get_searched_move_loc()].borrow_mut();
            position.do_move(&child.mov);
            position.board.invert();
            child.search(position);
        }

        let mut avg = 0.;
        let mut best = 0.;
        self.visits = 0;
        for c in self.children.iter() {
            let child = c.borrow();

            avg += child.score;
            self.visits += child.visits;

            if child.score > best {
                best = child.score;
            }
        }

        avg /= self.children.len() as f64;

        best = 1. - best;
        avg = 1. - avg;

        let trust = self.calc_trust();
        self.score = best * trust + avg * (1. - trust);

        for c in self.children.iter() {
            c.borrow_mut().update_ucb(self.visits);
        }
    }

    fn get_best_move_loc(&self) -> usize {
        let mut maxloc = 0;
        let mut max = 0.0;

        for i in 0..self.children.len() {
            let child = self.children[i].borrow();

            if child.score > max {
                max = child.score;
                maxloc = i;
            }
        }

        maxloc
    }

    pub fn do_move(&mut self, m: &Move) {
        let mut tmp = self.children[0].clone();
        for c in self.children.iter() {
            if c.borrow().mov == *m {
                tmp = c.clone();
            }
        }
        self.children = Vec::new();
        if tmp.borrow().mov == *m {
            *self = Rc::try_unwrap(tmp).ok().unwrap().into_inner();
            println!("{}", self.score);
        } else {
            *self = GameTree::new();
        }
    }

    pub fn get_best_move(&self) -> Move {
        self.children[self.get_best_move_loc()].borrow().mov.clone()
    }

    pub fn get_searched_move(&self) -> Move {
        self.children[self.get_searched_move_loc()].borrow().mov.clone()
    }

    pub fn get_best_line(&self) -> Vec<Move> {
        let mut out = Vec::new();
        let mut outer = Rc::new(RefCell::new(self.clone()));

        while !outer.borrow().children.is_empty() {
            let loc = outer.borrow().get_best_move_loc();

            let tmp = outer.borrow().clone().children[loc].clone();
            outer = tmp;
            print!("{} ", outer.borrow().score);
            out.push(outer.borrow().mov.clone());
        }
        println!();

        out
    }

    pub fn get_searched_line(&self) -> Vec<Move> {
        let mut out = Vec::new();
        let mut outer = Rc::new(RefCell::new(self.clone()));

        while !outer.borrow().children.is_empty() {
            let loc = outer.borrow().get_searched_move_loc();

            let tmp = outer.borrow().clone().children[loc].clone();
            outer = tmp;
            out.push(outer.borrow().mov.clone());
        }

        out
    }
}

use crate::eval::eval;

pub fn alphabeta(mut pos: Position,
                 mut alpha: i32,
                 beta: i32,
                 depth: usize) -> i32
{
    pos.set_moves();
    if let Some(n) = pos.test_endgame() {
        if n == 1 {
            return 0;
        } else {
            return -100000;
        }
    }

    if depth == 0 {return eval(&mut pos)}

    for m in pos.gen_moves() {
        let mut p = pos.clone();
        p.do_move(&m);
        p.board.invert();
        let score = -alphabeta(p, -beta, -alpha, depth - 1);
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    return alpha;
}

pub fn best_move(mut pos: Position, depth: usize, lastbest: Option<Move>)
    -> (Option<Move>, i32)
{
    pos.set_moves();
    let mut best_move = None;
    let mut best_score = -1000000;
    let mut moves = pos.gen_moves();
    if lastbest != None {
        let lastbest = lastbest.unwrap();
        moves.retain(|m| *m != lastbest);
        moves.push(lastbest);
    }
    for m in moves.into_iter().rev() {
        let mut p = pos.clone();
        p.do_move(&m);
        p.board.invert();
        let score = -alphabeta(p, -1000000, -best_score, depth - 1);
        if score > best_score {
            // println!("{:?} {}", m, score);
            best_move = Some(m);
            best_score = score;
        }
    }
    return (best_move, best_score);
}

use std::io::Write;
use std::time::{Duration, SystemTime};

pub fn ab_search(mut pos: Position, time: usize) -> (Option<Move>, i32) {
    let time = time as u128;
    let now = SystemTime::now();

    let mut best = None;
    let mut score = 0;;
    let mut d = 1;

    while now.elapsed().ok().unwrap().as_millis() < time {
        print!("{}", d % 10);
        std::io::stdout().flush();
        match best_move(pos.clone(), d, best) {
            (None, _) => return (None, score),
            (m, s) => {
                best = m;
                score = s
            }
        }
        d += 1;
    }
    println!();
    (best, score)
}
