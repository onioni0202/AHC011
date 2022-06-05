use proconio::input;
use proconio::marker::Chars;
use rand::prelude::*;
use std::collections::VecDeque;
const SEED: u128 = 0;
const SEARCH_TIME: [f32; 11] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.1, 0.2, 0.3, 0.4, 0.5];

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct Board {
    board_size: usize,
    board_list: Vec<u8>,
    empty_tile_area: (usize, usize),
}

#[allow(dead_code)]
impl Board {
    const DCHARS: [char; 4] = ['L', 'U', 'R', 'D'];
    const DH: [usize; 4] = [0, !0, 0, 1];
    const DW: [usize; 4] = [!0, 0, 1, 0]; //LURD
    fn new(board_size: usize, board_list: Vec<u8>) -> Board {
        let mut empty_tile_area = (0, 0);
        'mainloop: for h in 0..board_size {
            for w in 0..board_size {
                if board_list[h * board_size + w] == 0 {
                    empty_tile_area = (h, w);
                    break 'mainloop;
                }
            }
        }
        Board {
            board_size: board_size,
            board_list: board_list,
            empty_tile_area: empty_tile_area,
        }
    }

    fn from_dchar_to_didx(dchar: char) -> usize {
        match dchar {
            'L' => 0,
            'U' => 1,
            'R' => 2,
            'D' => 3,
            _ => unreachable!()
        }
    }

    fn update_empty_tile_area(&mut self) {
        'mainloop: for h in 0..self.board_size {
            for w in 0..self.board_size {
                if self.board_list[h * self.board_size + w] == 0 {
                    self.empty_tile_area = (h, w);
                    break 'mainloop;
                }
            }
        }
    }

    fn move_empty_tile(&mut self, dchar: char) -> bool {
        let didx = Board::from_dchar_to_didx(dchar);
        let (th, tw) = (
            self.empty_tile_area.0.wrapping_add(Self::DH[didx]),
            self.empty_tile_area.1.wrapping_add(Self::DW[didx]),
        );
        if th >= self.board_size || tw >= self.board_size {
            return false;
        }
        self.swap(self.empty_tile_area.0 * self.board_size + self.empty_tile_area.1 , 
                  th * self.board_size + tw);
        self.empty_tile_area = (th, tw);
        return true;
    }

    fn swap(&mut self, idx1: usize, idx2: usize) {
        self.board_list.swap(idx1, idx2);
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
            eprintln!("{}", bytes.into_iter().collect::<String>());
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
    let (max_iter, board) = input();
    let duration = 2.8;
    let start_time = std::time::Instant::now();
    let mut rng = rand_pcg::Pcg64Mcg::new(SEED);
    let mut iter_num = 0;
    let mut best_solution = vec![];
    let mut best_score = 0.0;
    loop {
        iter_num += 1;
        let mut now_board = board.clone();
        let limit_time = duration - (std::time::Instant::now() - start_time).as_secs_f32();
        if limit_time <= 0.0 {
            break;
        }
        let best_board = annealing_search_best_board(&now_board, max_iter, 
                                        f32::min(SEARCH_TIME[board.board_size], limit_time), &mut rng);
        let greedy_solution = greedy(&mut now_board, &best_board);
        if greedy_solution.len() > max_iter {
            continue;
        }
        let (now_score, now_solution) = annealing(&board, max_iter, greedy_solution, 0.15, &mut rng);
        if now_score > best_score {
            best_score = now_score;
            best_solution = now_solution;
        }
    }
    eprintln!("FINAL_BEST_SCORE = {}; ITER_NUM = {}", best_score, iter_num);
    println!("{}", best_solution.iter().collect::<String>());
}

fn greedy(board: &mut Board, best_board: &Board) -> Vec<char> {
    let board_size = board.board_size;
    let mut solution = vec![];
    let mut fixed = vec![false; board_size * board_size];

    'mainloop: for h in 0..board_size - 1 {
        for w in 0..board_size - 1 {
            if h < board_size - 2 && w < board_size - 2 {
                move_tile(board, best_board, (h, w), (h, w), &mut solution, &mut fixed);
                fixed[h * board_size + w] = true;
            } else if h < board_size - 2 && w == board_size - 2 {
                if !move_tile(board, best_board, (h, w + 1), (h, w), &mut solution, &mut fixed) {
                    break 'mainloop;
                }
                fixed[h * board_size + w] = true;
                if !move_tile(board, best_board, (h, w), (h + 1, w), &mut solution, &mut fixed) {
                    break 'mainloop;
                }
                fixed[(h + 1) * board_size + w] = true;
                if !move_empty_tile_to_goal(board, (h, w + 1), &mut solution, &fixed) {
                    break 'mainloop;
                }
                board.move_empty_tile('L');
                board.move_empty_tile('D');
                solution.push('L');
                solution.push('D');
                fixed[h * board_size + w] = true;
                fixed[h * board_size + (w + 1)] = true;
                fixed[(h + 1) * board_size + w] = false;
            } else {
                if !move_tile(board,best_board, (h + 1, w), (h, w), &mut solution, &mut fixed) {
                    break 'mainloop;
                }
                fixed[h * board_size + w] = true;
                if !move_tile(board,best_board, (h, w), (h, w + 1), &mut solution, &mut fixed) {
                    break 'mainloop;
                }
                fixed[h * board_size + w + 1] = true;
                if !move_empty_tile_to_goal(board, (h + 1, w), &mut solution, &fixed) {
                    break 'mainloop;
                }
                board.move_empty_tile('U');
                board.move_empty_tile('R');
                solution.push('U');
                solution.push('R');
                fixed[h * board_size + w] = true;
                fixed[(h + 1) * board_size + w] = true;
                fixed[h * board_size + w + 1] = false;
            }
        }
    }
    solution
}

fn move_tile(
    board: &mut Board,
    best_board: &Board,
    best_tile_pos: (usize, usize),
    goal: (usize, usize),
    solution: &mut Vec<char>,
    fixed: &mut Vec<bool>,
) -> bool {
    let board_size = board.board_size;
    let best_tile = best_board.get(best_tile_pos.0, best_tile_pos.1);
    if let Some(tile_positions) = search_tiles(&board, best_tile, goal, &fixed) {
        'mainloop: for mut tile_pos in tile_positions {
            if let Some(tile_path) = find_path(board, tile_pos, goal, fixed) {
                let mut board_copy = board.clone();
                let mut movement = vec![];
                for dchar in tile_path {
                    let didx = Board::from_dchar_to_didx(dchar);
                    fixed[tile_pos.0 * board_size + tile_pos.1] = true;
                    let nxt_tile_pos = (
                        tile_pos.0.wrapping_add(Board::DH[didx]),
                        tile_pos.1.wrapping_add(Board::DW[didx]),
                    );
                    if !move_empty_tile_to_goal(&mut board_copy, nxt_tile_pos, &mut movement, fixed) {
                        fixed[tile_pos.0 * board_size + tile_pos.1] = false;
                        continue 'mainloop;
                    }
                    movement.push(Board::DCHARS[(didx + 2) % 4]);
                    board_copy.move_empty_tile(Board::DCHARS[(didx + 2) % 4]);
                    fixed[tile_pos.0 * board_size + tile_pos.1] = false;
                    tile_pos = nxt_tile_pos;
                }
                for dchar in movement {
                    solution.push(dchar);
                    board.move_empty_tile(dchar);
                }
                return true;
            }
        }
    }
    return false;
}

fn move_empty_tile_to_goal(
    board: &mut Board,
    goal: (usize, usize),
    solution: &mut Vec<char>,
    fixed: &Vec<bool>,
) -> bool {
    if goal == board.empty_tile_area {
        return true;
    }
    if let Some(empty_tile_path) = find_path(board, board.empty_tile_area, goal, fixed) {
        for dchar in empty_tile_path {
            solution.push(dchar);
            board.move_empty_tile(dchar);
        }
        return true;
    }
    return false;
    
}

fn search_tiles(board: &Board, tile: u8, start: (usize, usize), fixed: &Vec<bool>) -> Option<Vec<(usize, usize)>> {
    let board_size = board.board_size;
    let mut que = VecDeque::new();
    let mut seen = vec![false; board_size * board_size];
    que.push_back(start);
    seen[start.0 * board_size + start.1] = true;
    let mut positions = vec![];
    while let Some((h_now, w_now)) = que.pop_front() {
        if board.get(h_now, w_now) == tile {
            positions.push((h_now, w_now));
        }
        for didx in 0..4 {
            let (h_to, w_to) = (
                h_now.wrapping_add(Board::DH[didx]),
                w_now.wrapping_add(Board::DW[didx]),
            );
            if h_to >= board_size || w_to >= board_size {
                continue;
            }
            if !fixed[h_to * board_size + w_to] && !seen[h_to * board_size + w_to] {
                seen[h_to * board_size + w_to] = true;
                que.push_back((h_to, w_to));
            }
        }
    }
    if positions.len() > 0 {
        return Some(positions);
    }
    return None;
}

fn find_path(
    board: &Board,
    start: (usize, usize),
    goal: (usize, usize),
    fixed: &Vec<bool>,
) -> Option<Vec<char>> {
    let board_size = board.board_size;
    let mut que = VecDeque::new();
    let mut dist = vec![std::i32::MAX; board_size * board_size];
    let mut prev = vec![None; board_size * board_size];
    que.push_back(start);
    dist[start.0 * board_size + start.1] = 0;
    while let Some((h_now, w_now)) = que.pop_front() {
        if (h_now, w_now) == goal {
            break;
        }
        for didx in 0..4 {
            let (h_to, w_to) = (
                h_now.wrapping_add(Board::DH[didx]),
                w_now.wrapping_add(Board::DW[didx]),
            );
            if h_to >= board_size || w_to >= board_size {
                continue;
            }
            if fixed[h_to * board_size + w_to] {
                continue;
            }
            if dist[h_to * board_size + w_to] > dist[h_now * board_size + w_now] + 1 {
                dist[h_to * board_size + w_to] = dist[h_now * board_size + w_now] + 1;
                prev[h_to * board_size + w_to] = Some(((h_now, w_now), Board::DCHARS[didx]));
                que.push_back((h_to, w_to));
            }
        }
    }
    if prev[goal.0 * board_size + goal.1].is_none() {
        return None;
    }
    let mut path = vec![];
    let mut now_pos = goal;
    while let Some((nxt_pos, dchar)) = prev[now_pos.0 * board_size + now_pos.1] {
        now_pos = nxt_pos;
        path.push(dchar);
    }
    path.reverse();
    Some(path)
}

fn annealing_search_best_board(
    init_board: &Board, 
    max_iter: usize, 
    duration: f32, 
    rng: &mut rand_pcg::Pcg64Mcg
) -> Board {
    const START_TEMP: f32 = 500.0;
    const END_TEMP: f32 = 5.0;
    let start_time = std::time::Instant::now();
    let board_size = init_board.board_size;
    let mut board = init_board.clone();
    let empty_tile_position = board_size * board_size - 1;
    board.swap(board.empty_tile_area.0 * board_size + board.empty_tile_area.1, empty_tile_position);
    board.update_empty_tile_area();
    let mut score = calc_score(&board, 0, max_iter);
    let mut best_board = board.clone();
    let mut best_score = score;
    let mut iter_num = 0;
    loop {
        let diff_time = (std::time::Instant::now() - start_time).as_secs_f32();
        iter_num += 1;
        if diff_time > duration {
            break;
        }
        let choice1 = rng.gen_range(0, board_size * board_size - 2);
        let choice2 = rng.gen_range(choice1 + 1, board_size * board_size - 1);
        if choice1 == choice2 {
            continue;
        }
        board.swap(choice1, choice2);
        let new_score = calc_score(&board, 0, max_iter);
        let temp = START_TEMP + (END_TEMP - START_TEMP) * diff_time / duration;
        if new_score > best_score {
            best_score = new_score;
            best_board = board.clone();
            if best_score > 5e5 {
                break;
            }
        }
        if f32::exp((new_score - score) as f32 / temp) > rng.gen() {
            score = new_score;
        } else {
            board.swap(choice1, choice2);
        }
    }
    eprintln!("search iter num = {}, best score = {}", iter_num, best_score);
    best_board.update_empty_tile_area();
    best_board
}

fn annealing(
    board: &Board,
    max_iter: usize,
    movement: Vec<char>,
    duration: f32,
    rng: &mut rand_pcg::Pcg64Mcg,
) -> (f32, Vec<char>) {
    const START_TEMP: f32 = 1000.0;
    const END_TEMP: f32 = 5.0;
    let start_time = std::time::Instant::now();
    let mut solution = movement.clone();
    let mut score = calc_score(&board, solution.len(), max_iter);
    let mut best_solution = movement.clone();
    let mut best_score = score;
    let mut iter_num = 0;
    'mainloop: loop {
        iter_num += 1;
        let diff_time = (std::time::Instant::now() - start_time).as_secs_f32();
        if diff_time > duration {
            break;
        }
        let mut new_board = board.clone();
        let mut new_solution = solution.clone();
        let selection;
        if iter_num == 1 {
            selection = 6;
        } else {
            selection = rng.gen_range(0, 6);
        }
        match selection {
            0 => {
                if new_solution.len() < max_iter / 2 {
                    continue;
                }
                let select1 = rng.gen_range(0, new_solution.len() - 1);
                let select2 = rng.gen_range(select1 + 1, new_solution.len());
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
                if new_solution.len() == 0 || new_solution.len() >= max_iter {
                    continue;
                }
                let select = rng.gen_range(0, new_solution.len());
                let random_dchar = Board::DCHARS[rng.gen_range(0, 4)];
                new_solution.insert(select, random_dchar);
            }
            4 => {
                if new_solution.len() >= max_iter {
                    continue;
                }
                let random_dchar = Board::DCHARS[rng.gen_range(0, 4)];
                new_solution.push(random_dchar);
            }
            5 => {
                new_solution.pop();
            }
            6 => {
                // pass
            }
            _ => unreachable!(),
        }
        for &dchar in &new_solution {
            if !new_board.move_empty_tile(dchar) {
                continue 'mainloop;
            }
        }
        let new_score = calc_score(&new_board, new_solution.len(), max_iter);
        if iter_num == 1 && new_score < 4.5e5 {
            return (new_score, new_solution);
        }
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
    (best_score, best_solution)
}

fn calc_score(board: &Board, iter: usize, max_iter: usize) -> f32 {
    let board_size = board.board_size;
    let mut passed = vec![false; board_size * board_size];
    let mut que = VecDeque::new();
    let mut max_tree_size = 0;
    let mut score = 0.0;
    for h_st in 0..board_size {
        for w_st in 0..board_size {
            if !passed[h_st * board_size + w_st] {
                let mut tree_size = 0;
                passed[h_st * board_size + w_st] = true;
                que.push_back(((h_st, w_st), 10));
                while let Some(((h_now, w_now), prev)) = que.pop_front() {
                    tree_size += 1;
                    for didx in 0..4 {
                        if prev == (didx + 2) % 4 {
                            continue;
                        }
                        if ((board.get(h_now, w_now) >> didx) & 1) == 1 {
                            let (h_to, w_to) = (
                                h_now.wrapping_add(Board::DH[didx]),
                                w_now.wrapping_add(Board::DW[didx]),
                            );
                            if h_to >= board_size || w_to >= board_size {
                                continue;
                            }
                            if ((board.get(h_to, w_to) >> ((didx + 2) % 4)) & 1) == 1 {
                                if !passed[h_to * board_size + w_to] {
                                    passed[h_to * board_size + w_to] = true;
                                    que.push_back(((h_to, w_to), didx));
                                }
                            }
                        }
                    }
                }
                max_tree_size = i32::max(max_tree_size, tree_size);
            }
        }
    }
    if max_tree_size == (board_size * board_size - 1) as i32 {
        score += 5e5 * (2.0 - (iter as f32 / max_iter as f32));
    } else {
        score += 5e5 * (max_tree_size as f32) / (board_size * board_size - 1) as f32;
    }
    score
}
