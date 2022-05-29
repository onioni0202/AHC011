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
    from_position: Option<(usize, usize)>,
    from_dchar: Option<char>,
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
            from_position: None,
            from_dchar: None,
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
        self.from_position = Some(self.empty_block_area);
        self.from_dchar = Some(Board::DCHARS[didx]);
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
    let init_movement = vec![];
    let best_solution = annealing(max_iter, &init_board, init_movement, 2.98);
    println!("{}", best_solution.iter().collect::<String>())
}

fn annealing(max_iter: usize, board: &Board, movement: Vec<char>, duration: f32) -> Vec<char> {
    const START_TEMP: f32 = 2000.0;
    const END_TEMP: f32 = 5.0;
    let start_time = std::time::Instant::now();
    let mut solution = movement.clone();
    let mut score = calc_score(&board);
    let mut best_solution = movement.clone();
    let mut best_score = score;
    let mut rng = rand_pcg::Pcg64Mcg::new(SEED);
    let mut iter_num = 0;
    'mainloop: loop {
        iter_num += 1;
        let diff_time = (std::time::Instant::now() - start_time).as_secs_f32();
        if diff_time > duration {
            break;
        }
        let mut new_board = board.clone();
        let mut new_solution = solution.clone();
        let selection: usize = rng.gen_range(0, 5);
        match selection {
            0 => {
                if new_solution.len() < max_iter / 2 {
                    continue;
                }
                let select1 = rng.gen_range(0, new_solution.len());
                let select2 = rng.gen_range(0, new_solution.len());
                new_solution.swap(select1, select2);
            }
            1 => {
                if new_solution.len() == 0 {
                    continue;
                }
                let select = rng.gen_range(0, new_solution.len());
                let random_dchar = Board::DCHARS[rng.gen_range(0, 4)];
                new_solution[select] = random_dchar;
            }
            2 => {
                if new_solution.len() < max_iter / 2 {
                    continue;
                }
                let select = rng.gen_range(0, new_solution.len());
                new_solution.remove(select);
            }
            3 => {
                if new_solution.len() == 0 || new_solution.len() == max_iter {
                    continue;
                }
                let select = rng.gen_range(0, new_solution.len());
                let random_dchar = Board::DCHARS[rng.gen_range(0, 4)];
                new_solution.insert(select, random_dchar);
            }
            4 => {
                if new_solution.len() == max_iter {
                    continue;
                }
                let random_dchar = Board::DCHARS[rng.gen_range(0, 4)];
                new_solution.push(random_dchar);
            }
            _ => unreachable!(),
        }
        for &dchar in &new_solution {
            match dchar {
                'L' => {
                    if !new_board.move_left() {
                        continue 'mainloop;
                    };
                }
                'U' => {
                    if !new_board.move_up() {
                        continue 'mainloop;
                    };
                }
                'D' => {
                    if !new_board.move_down() {
                        continue 'mainloop;
                    };
                }
                'R' => {
                    if !new_board.move_right() {
                        continue 'mainloop;
                    };
                }
                _ => unreachable!(),
            }
        }
        let new_score = calc_score(&new_board);
        let temp = START_TEMP + (END_TEMP - START_TEMP) * diff_time / duration;
        if f32::exp((new_score - score) / temp) > rng.gen() {
            score = new_score;
            solution = new_solution.clone();
        }
        if new_score > best_score {
            best_score = new_score;
            best_solution = solution.clone();
        }
    }
    eprintln!("BEST SCORE = {}", best_score);
    eprintln!("ITER = {}", iter_num);
    best_solution
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
