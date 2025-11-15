// src/game.rs

use crate::board::{Board, Owner};
use crate::piece::Piece;
use std::collections::VecDeque;

pub struct Game {
    pub my_player: u8,
}

impl Game {
    pub fn new(my_player: u8) -> Self {
        Game { my_player }
    }

    /// Try all valid placements and choose one with the best score.
    /// Returns (row, col) in board coordinates (y, x).
    pub fn choose_best_move(&self, board: &Board, piece: &Piece) -> Option<(usize, usize)> {
        if piece.cells.is_empty() || board.rows == 0 || board.cols == 0 {
            return None;
        }

        if piece.height > board.rows || piece.width > board.cols {
            return None;
        }

        let mut best_pos: Option<(usize, usize)> = None;
        let mut best_score: i64 = i64::MAX;

        for top_y in 0..=board.rows - piece.height {
            for left_x in 0..=board.cols - piece.width {
                if !self.is_valid_placement(board, piece, top_y, left_x) {
                    continue;
                }

                let score = self.score_placement(board, piece, top_y, left_x);

                if score < best_score {
                    best_score = score;
                    best_pos = Some((top_y, left_x));
                } else if score == best_score {
                    // Tie-breaker: smallest row, then smallest col (deterministic).
                    if let Some((by, bx)) = best_pos {
                        if top_y < by || (top_y == by && left_x < bx) {
                            best_pos = Some((top_y, left_x));
                        }
                    }
                }
            }
        }

        best_pos
    }

    /// Valid placement:
    /// - piece must stay inside board
    /// - cannot overlap opponent
    /// - must overlap OWN territory exactly once
    fn is_valid_placement(
        &self,
        board: &Board,
        piece: &Piece,
        top_y: usize,
        left_x: usize,
    ) -> bool {
        let mut overlap_count = 0;

        for &(dy, dx) in &piece.cells {
            let y = top_y + dy;
            let x = left_x + dx;

            if y >= board.rows || x >= board.cols {
                return false;
            }

            match board.cells[y][x] {
                Owner::Opponent => {
                    // Cannot place on opponent cell.
                    return false;
                }
                Owner::Me => {
                    overlap_count += 1;
                    if overlap_count > 1 {
                        return false;
                    }
                }
                Owner::Empty => {}
            }
        }

        overlap_count == 1
    }

    /// Core heuristic focused on:
    /// - minimizing enemy's future space ("reachable empty area")
    /// - maximizing our future space
    /// - maximizing immediate gain (new cells we take now)
    ///
    /// LOWER score is better.
    fn score_placement(
        &self,
        board: &Board,
        piece: &Piece,
        top_y: usize,
        left_x: usize,
    ) -> i64 {
        // 1) Count how many new cells we gain right now.
        let mut new_cells: u64 = 0;
        for &(dy, dx) in &piece.cells {
            let y = top_y + dy;
            let x = left_x + dx;
            if board.cells[y][x] == Owner::Empty {
                new_cells += 1;
            }
        }

        // 2) Approximate how much free space the enemy can still reach after this move.
        let enemy_space = self.reachable_space_for_enemy(board, piece, top_y, left_x);

        // 3) Approximate how much free space WE can reach after this move.
        let my_space = self.reachable_space_for_me(board, piece, top_y, left_x);

        // Main idea:
        //  - heavily penalize large enemy_space
        //  - reward large my_space
        //  - reward immediate new_cells
        //
        // You can tweak these weights, but this is a good starting point.
        let score =
            (enemy_space as i64) * 1000  // biggest priority: starve enemy
            - (my_space as i64) * 400    // also keep our options open
            - (new_cells as i64) * 40;   // immediate gain

        score
    }

    /// Helper: get the "owner" of (y, x) as if the piece is already placed.
    /// If (y, x) is covered by the piece, treat it as Owner::Me.
    fn owner_with_piece(
        &self,
        board: &Board,
        piece: &Piece,
        top_y: usize,
        left_x: usize,
        y: usize,
        x: usize,
    ) -> Owner {
        for &(dy, dx) in &piece.cells {
            if top_y + dy == y && left_x + dx == x {
                return Owner::Me;
            }
        }
        board.cells[y][x]
    }

    /// BFS the board from all enemy cells and count distinct empty cells
    /// they can still reach if our piece is placed.
    fn reachable_space_for_enemy(
        &self,
        board: &Board,
        piece: &Piece,
        top_y: usize,
        left_x: usize,
    ) -> u64 {
        let rows = board.rows;
        let cols = board.cols;
        let mut visited = vec![false; rows * cols];
        let mut q: VecDeque<(usize, usize)> = VecDeque::new();

        // Initialize queue with all enemy cells (after placing our piece).
        for y in 0..rows {
            for x in 0..cols {
                let owner = self.owner_with_piece(board, piece, top_y, left_x, y, x);
                if owner == Owner::Opponent {
                    let idx = y * cols + x;
                    if !visited[idx] {
                        visited[idx] = true;
                        q.push_back((y, x));
                    }
                }
            }
        }

        let mut reachable_empty: u64 = 0;

        const DIRS: &[(isize, isize)] = &[(1, 0), (-1, 0), (0, 1), (0, -1)];

        while let Some((y, x)) = q.pop_front() {
            for &(dy, dx) in DIRS {
                let ny_i = y as isize + dy;
                let nx_i = x as isize + dx;

                if ny_i < 0 || nx_i < 0 {
                    continue;
                }

                let ny = ny_i as usize;
                let nx = nx_i as usize;

                if ny >= rows || nx >= cols {
                    continue;
                }

                let idx = ny * cols + nx;
                if visited[idx] {
                    continue;
                }

                let owner = self.owner_with_piece(board, piece, top_y, left_x, ny, nx);

                match owner {
                    Owner::Opponent => {
                        // Enemy territory: they can pass through.
                        visited[idx] = true;
                        q.push_back((ny, nx));
                    }
                    Owner::Empty => {
                        // Empty cell they could potentially occupy.
                        visited[idx] = true;
                        reachable_empty += 1;
                        q.push_back((ny, nx));
                    }
                    Owner::Me => {
                        // Our territory (including the new piece) blocks their path.
                    }
                }
            }
        }

        reachable_empty
    }

    /// BFS the board from all our cells and count distinct empty cells
    /// WE can reach after the move.
    fn reachable_space_for_me(
        &self,
        board: &Board,
        piece: &Piece,
        top_y: usize,
        left_x: usize,
    ) -> u64 {
        let rows = board.rows;
        let cols = board.cols;
        let mut visited = vec![false; rows * cols];
        let mut q: VecDeque<(usize, usize)> = VecDeque::new();

        // Initialize queue with all our cells (after placing our piece).
        for y in 0..rows {
            for x in 0..cols {
                let owner = self.owner_with_piece(board, piece, top_y, left_x, y, x);
                if owner == Owner::Me {
                    let idx = y * cols + x;
                    if !visited[idx] {
                        visited[idx] = true;
                        q.push_back((y, x));
                    }
                }
            }
        }

        let mut reachable_empty: u64 = 0;
        const DIRS: &[(isize, isize)] = &[(1, 0), (-1, 0), (0, 1), (0, -1)];

        while let Some((y, x)) = q.pop_front() {
            for &(dy, dx) in DIRS {
                let ny_i = y as isize + dy;
                let nx_i = x as isize + dx;

                if ny_i < 0 || nx_i < 0 {
                    continue;
                }

                let ny = ny_i as usize;
                let nx = nx_i as usize;

                if ny >= rows || nx >= cols {
                    continue;
                }

                let idx = ny * cols + nx;
                if visited[idx] {
                    continue;
                }

                let owner = self.owner_with_piece(board, piece, top_y, left_x, ny, nx);

                match owner {
                    Owner::Me => {
                        // Our own territory: we can pass through.
                        visited[idx] = true;
                        q.push_back((ny, nx));
                    }
                    Owner::Empty => {
                        // Empty cell we can reach/occupy in future.
                        visited[idx] = true;
                        reachable_empty += 1;
                        q.push_back((ny, nx));
                    }
                    Owner::Opponent => {
                        // Opponent territory blocks our path.
                    }
                }
            }
        }

        reachable_empty
    }
}
