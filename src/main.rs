use proconio::input;
use proconio::marker::Chars;
use rand::prelude::*;
use std::collections::VecDeque;

const SEED: u128 = 0;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Board {
    board_size: usize,
    board_list: Vec<u8>,
    empty_block_area: (usize, usize),
    from_where: Option<usize>,
}

#[allow(dead_code)]
impl Board {
    const DINDEX: [char; 4] = ['L', 'U', 'R', 'D'];
    const DH: [usize; 4] = [0, !0, 0, 1];
    const DW: [usize; 4] = [!0, 0, 1, 0]; //LURD
    fn new(board_size: usize, board_list: Vec<u8>) -> Board {
        let mut empty_block_area = (0, 0);
        for h in 0..board_size {
            for w in 0..board_size {
                if board_list[h * board_size + w] == 0 {
                    empty_block_area = (h, w);
                }
            }
        }
        Board {
            board_size: board_size,
            board_list: board_list,
            empty_block_area: empty_block_area,
            from_where: None,
        }
    }

    fn move_empty_block(&mut self, didx: usize) -> bool {
        let (th, tw) = (
            self.empty_block_area.0.wrapping_add(Self::DH[didx]),
            self.empty_block_area.1.wrapping_add(Self::DW[didx]),
        );
        if th >= self.board_size || tw >= self.board_size {
            return false;
        }
        self.swap(self.empty_block_area, (th, tw));
        self.empty_block_area = (th, tw);
        return true;
    }

    fn move_left(&mut self) -> bool {
        self.move_empty_block(0)
    }

    fn move_up(&mut self) -> bool {
        self.move_empty_block(1)
    }

    fn move_right(&mut self) -> bool {
        self.move_empty_block(2)
    }

    fn move_down(&mut self) -> bool {
        self.move_empty_block(3)
    }

    fn swap(&mut self, (h1, w1): (usize, usize), (h2, w2): (usize, usize)) {
        self.board_list
            .swap(h1 * self.board_size + w1, h2 * self.board_size + w2);
    }

    fn replace(&mut self, h: usize, w: usize, value: u8) {
        self.board_list[h * self.board_size + w] = value;
    }

    fn get(&self, h: usize, w: usize) -> u8 {
        self.board_list[h * self.board_size + w]
    }

    fn print_board(&self) {
        for h in 0..self.board_size {
            let mut bytes = vec![];
            for w in 0..self.board_size {
                bytes.push(format!("{:x}", self.board_list[h * self.board_size + w]));
            }
            println!("{}", bytes.into_iter().collect::<String>());
        }
    }
}

fn input() -> (usize, Board) {
    input! {
        board_size: usize,
        max_iter: usize,
        board_hex: [Chars; board_size]
    }
    let mut init_board_list = vec![0; board_size * board_size];
    for h in 0..board_size {
        for w in 0..board_size {
            if let Ok(num) = u8::from_str_radix(&board_hex[h][w].to_string(), 16) {
                init_board_list[h * board_size + w] = num;
            }
        }
    }
    let init_board = Board::new(board_size, init_board_list);
    (max_iter, init_board)
}

fn main() {
    let (max_iter, init_board) = input();
    let score = calc_score(&init_board);
    let best_board = annealing(&init_board, 1.0);
    best_board.print_board();
}

fn annealing(init_board: &Board, duration: f32) -> Board {
    const START_TEMP: f32 = 200.0;
    const END_TEMP: f32 = 5.0;
    let start_time = std::time::Instant::now();
    let board_size = init_board.board_size;
    let mut board = init_board.clone();
    let mut score = calc_score(&board);
    let mut best_board = board.clone();
    let mut best_score = score;
    let mut rng = rand_pcg::Pcg64Mcg::new(SEED);
    let mut iter_num = 0;
    loop {
        iter_num += 1;
        let diff_time = (std::time::Instant::now() - start_time).as_secs_f32();
        if diff_time > duration {
            break;
        }
        let mut new_board = board.clone();
        let choice1 = (rng.gen_range(0, board_size), rng.gen_range(0, board_size));
        let choice2 = (rng.gen_range(0, board_size), rng.gen_range(0, board_size));
        if choice1 == choice2 {
            continue;
        }
        new_board.swap(choice1, choice2);
        let new_score = calc_score(&new_board);
        let temp = START_TEMP + (END_TEMP - START_TEMP) * diff_time / duration;
        if f32::exp((new_score - score) / temp) > rng.gen() {
            score = new_score;
            board = new_board.clone();
        }
        if new_score > best_score {
            best_score = new_score;
            best_board = new_board;
            if best_score > 4.99e5 {
                break;
            }
        }
    }
    eprintln!("BEST SCORE = {}", best_score);
    eprintln!("ITER = {}", iter_num);
    best_board
}

fn calc_score(board: &Board) -> f32 {
    let board_size = board.board_size;
    let mut seen = vec![vec![false; board_size]; board_size];
    let mut que = VecDeque::new();
    let mut max_tree_size = 0;
    for h_st in 0..board_size {
        for w_st in 0..board_size {
            if !seen[h_st][w_st] {
                let mut tree_size = 0;
                seen[h_st][w_st] = true;
                que.push_back((h_st, w_st));
                while let Some((h_now, w_now)) = que.pop_front() {
                    tree_size += 1;
                    for didx in 0..4 {
                        if ((board.get(h_now, w_now) >> didx) & 1) == 1 {
                            let (h_to, w_to) = (
                                h_now.wrapping_add(Board::DH[didx]),
                                w_now.wrapping_add(Board::DW[didx]),
                            );
                            if h_to >= board_size || w_to >= board_size {
                                continue;
                            }
                            if ((board.get(h_to, w_to) >> ((didx + 2) % 4)) & 1) == 1
                                && !seen[h_to][w_to]
                            {
                                seen[h_to][w_to] = true;
                                que.push_back((h_to, w_to));
                            }
                        }
                    }
                }
                max_tree_size = i32::max(max_tree_size, tree_size);
            }
        }
    }
    5e5 * max_tree_size as f32 / (board_size * board_size - 1) as f32
}
