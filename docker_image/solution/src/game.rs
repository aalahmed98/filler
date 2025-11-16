// src/game.rs

use crate::board::{Board, Owner};
use crate::piece::Piece;

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
            eprintln!("[DEBUG] Board or piece empty");
            return None;
        }

        if piece.height > board.rows || piece.width > board.cols {
            eprintln!("[DEBUG] Piece too large: {}x{} vs board {}x{}", 
                piece.height, piece.width, board.rows, board.cols);
            return None;
        }

        // Precompute coordinates
        let mut enemy_coords: Vec<(usize, usize)> = Vec::new();
        let mut my_coords: Vec<(usize, usize)> = Vec::new();
        for y in 0..board.rows {
            for x in 0..board.cols {
                match board.cells[y][x] {
                    Owner::Opponent => enemy_coords.push((y, x)),
                    Owner::Me => my_coords.push((y, x)),
                    _ => {}
                }
            }
        }

        // For large boards, limit search space to area around our territory
        let (search_min_y, search_max_y, search_min_x, search_max_x) = if my_coords.is_empty() {
            (0, board.rows, 0, board.cols)
        } else if board.rows * board.cols > 2000 {
            // Large board optimization: search only near our territory
            let margin = 15.max(piece.height * 2).max(piece.width * 2);
            let min_y = my_coords.iter().map(|(y, _)| *y).min().unwrap().saturating_sub(margin);
            let max_y = (my_coords.iter().map(|(y, _)| *y).max().unwrap() + margin + piece.height).min(board.rows);
            let min_x = my_coords.iter().map(|(_, x)| *x).min().unwrap().saturating_sub(margin);
            let max_x = (my_coords.iter().map(|(_, x)| *x).max().unwrap() + margin + piece.width).min(board.cols);
            (min_y, max_y, min_x, max_x)
        } else {
            (0, board.rows, 0, board.cols)
        };

        let mut best_pos: Option<(usize, usize)> = None;
        let mut best_score: i64 = i64::MIN;

        // Optimized search within bounded area
        let max_y = search_max_y.saturating_sub(piece.height).saturating_add(1);
        let max_x = search_max_x.saturating_sub(piece.width).saturating_add(1);
        
        for top_y in search_min_y..max_y {
            for left_x in search_min_x..max_x {
                if !self.is_valid_placement(board, piece, top_y, left_x) {
                    continue;
                }

                let score = self.evaluate_placement(board, piece, top_y, left_x, &enemy_coords);

                if score > best_score {
                    best_score = score;
                    best_pos = Some((top_y, left_x));
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

    /// Simplified aggressive heuristic:
    /// Priority: Expand territory, move toward enemy, block opponent, maintain mobility
    ///
    /// HIGHER score is better.
    fn evaluate_placement(
        &self,
        board: &Board,
        piece: &Piece,
        top_y: usize,
        left_x: usize,
        enemy_coords: &[(usize, usize)],
    ) -> i64 {
        let rows = board.rows;
        let cols = board.cols;

        let mut new_territory: i64 = 0;
        let mut enemy_adjacency: i64 = 0;
        let mut future_liberties: i64 = 0;
        let mut blocked_opponent: i64 = 0;

        const DIRS: &[(isize, isize)] = &[(1, 0), (-1, 0), (0, 1), (0, -1)];

        for &(dy, dx) in &piece.cells {
            let ay = top_y + dy;
            let ax = left_x + dx;

            // Count new territory
            if board.cells[ay][ax] == Owner::Empty {
                new_territory += 1;
            }

            // Check all neighbors
            let mut empty_count = 0;
            let mut enemy_count = 0;
            
            for &(dyy, dxx) in DIRS {
                let ny_i = ay as isize + dyy;
                let nx_i = ax as isize + dxx;

                if ny_i < 0 || nx_i < 0 {
                    continue;
                }

                let ny = ny_i as usize;
                let nx = nx_i as usize;

                if ny >= rows || nx >= cols {
                    continue;
                }

                match board.cells[ny][nx] {
                    Owner::Empty => {
                        empty_count += 1;
                    }
                    Owner::Opponent => {
                        enemy_count += 1;
                    }
                    _ => {}
                }
            }

            // Reward being near enemy (for blocking)
            enemy_adjacency += enemy_count;
            
            // Reward having empty neighbors (mobility)
            future_liberties += empty_count;
            
            // Reward blocking enemy liberties
            if board.cells[ay][ax] == Owner::Empty && enemy_count > 0 {
                blocked_opponent += 1;
            }
        }

        // Calculate distance to nearest enemy
        let min_dist_sq = self.min_distance_to_enemy(piece, top_y, left_x, enemy_coords);
        
        // Aggressive closeness score - heavily reward being close to enemy
        let closeness_bonus = if min_dist_sq > 0 {
            10000 / (min_dist_sq as i64 + 1)
        } else {
            10000
        };

        // Simple, aggressive scoring:
        // 1. New territory is most important
        // 2. Stay close to enemy to compete for space
        // 3. Block opponent when possible  
        // 4. Maintain mobility to avoid being trapped
        let score =
            new_territory * 1000              // Expand!
            + closeness_bonus * 5             // Get close to enemy
            + blocked_opponent * 300          // Block when adjacent
            + enemy_adjacency * 50            // Reward being near enemy
            + future_liberties * 100;         // Keep options open

        score
    }

    /// Minimal squared distance from any newly placed piece cell
    /// to any enemy coordinate (on the current board).
    /// If there is no enemy, we return 0.
    fn min_distance_to_enemy(
        &self,
        piece: &Piece,
        top_y: usize,
        left_x: usize,
        enemy_coords: &[(usize, usize)],
    ) -> u64 {
        if enemy_coords.is_empty() {
            return 0;
        }

        let mut best: u64 = u64::MAX;

        for &(dy, dx) in &piece.cells {
            let y = top_y + dy;
            let x = left_x + dx;

            for &(ey, ex) in enemy_coords {
                let dy_i = ey as isize - y as isize;
                let dx_i = ex as isize - x as isize;
                let d = (dy_i * dy_i + dx_i * dx_i) as u64;
                if d < best {
                    best = d;
                }
            }
        }

        if best == u64::MAX { 0 } else { best }
    }
}
