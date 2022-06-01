use proconio::input;
use proconio::marker::Chars;
use rand::prelude::*;
use std::collections::{BinaryHeap, HashMap, VecDeque};

const SEED: u128 = 0;
const BEAM_WIDTH: [usize; 11] = [0, 0, 0, 0, 0, 0, 3400, 1700, 950, 690, 450];

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Board {
    board_size: usize,
    board_list: Vec<u8>,
    empty_block_area: (usize, usize),
}

#[allow(dead_code)]
impl Board {
    const DCHARS: [char; 4] = ['L', 'U', 'R', 'D'];
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

struct Node {
    board: Board,
    score: i32
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.cmp(&other.score)
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
    let best_solution = beam_search(max_iter, init_board, 2.98);
    println!("{}", best_solution.iter().collect::<String>());
}

fn beam_search(max_iter: usize, init_board: Board, duration: f32) -> Vec<char> {
    let beam_width = BEAM_WIDTH[init_board.board_size];
    let start_time = std::time::Instant::now();
    let mut rng = rand_pcg::Mcg128Xsl64::new(SEED);
    let mut best_score = calc_score(&init_board);
    let mut best_board = init_board.clone();
    let mut record = HashMap::new();
    let mut que = vec![];
    que.push(Node {
        board: init_board.clone(),
        score: calc_score(&init_board)
    });
    record.insert(init_board, 'S');
    'mainloop: for _ in 0..max_iter {
        let mut next_que = vec![];
        while let Some(mut node) = que.pop() {
            let diff_time = (std::time::Instant::now() - start_time).as_secs_f32();
            if diff_time > duration {
                break 'mainloop
            }
            if node.score > best_score {
                best_score = node.score;
                best_board = node.board.clone();
            }
            // up
            if node.board.move_up() {
                if !record.contains_key(&node.board) {
                    record.insert(node.board.clone(), 'U');
                    next_que.push(Node {
                        board: node.board.clone(),
                        score: calc_score(&node.board) + rng.gen_range(0, 1000)
                    });
                }
                node.board.move_down();
            }
            // down
            if node.board.move_down() {
                if !record.contains_key(&node.board) {
                    record.insert(node.board.clone(), 'D');
                    next_que.push(Node {
                        board: node.board.clone(),
                        score: calc_score(&node.board) + rng.gen_range(0, 1000)
                    });
                }
                node.board.move_up();
            }
            // left
            if node.board.move_left() {
                if !record.contains_key(&node.board) {
                    record.insert(node.board.clone(), 'L');
                    next_que.push(Node {
                        board: node.board.clone(),
                        score: calc_score(&node.board) + rng.gen_range(0, 1000)
                    });
                }
                node.board.move_right();
            }
            // right
            if node.board.move_right() {
                if !record.contains_key(&node.board) {
                    record.insert(node.board.clone(), 'R');
                    next_que.push(Node {
                        board: node.board.clone(),
                        score: calc_score(&node.board) + rng.gen_range(0, 1000)
                    });
                }
                node.board.move_left();
            }
        }
        next_que.sort();
        for _ in 0..beam_width {
            if let Some(v) = next_que.pop() {
                que.push(v);
            }
        }
    }
    let mut best_solution = vec![];
    while let Some(&dchar) = record.get(&best_board) {
        if dchar == 'S' {
            break;
        }
        best_solution.push(dchar);
        match dchar {
            'U' => {
                best_board.move_down();
            }
            'D' => {
                best_board.move_up();
            }
            'L' => {
                best_board.move_right();
            }
            'R' => {
                best_board.move_left();
            }
            'S' => {
                break;
            }
            _ => unreachable!(),
        }
    }
    eprintln!("score = {}", best_score);
    best_solution.reverse();
    best_solution
}

fn calc_score(board: &Board) -> i32 {
    let mut rng = rand_pcg::Pcg64Mcg::new(SEED);
    let board_size = board.board_size;
    let mut score = 0;
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
                                score -= 1000;
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
    score += (5e5 * max_tree_size as f32 / (board_size * board_size - 1) as f32).round() as i32;
    score + rng.gen_range(0, 1000)
}


