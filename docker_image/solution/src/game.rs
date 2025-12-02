// src/game.rs
// Aggressive blocking strategy: Rush to enemy, block them, take the rest

use crate::board::{Board, Owner};
use crate::piece::Piece;

pub struct Game {
    pub my_player: u8,
}

impl Game {
    pub fn new(my_player: u8) -> Self {
        Game { my_player }
    }

    pub fn choose_best_move(&self, board: &Board, piece: &Piece) -> Option<(usize, usize)> {
        if piece.cells.is_empty() || board.rows == 0 || board.cols == 0 {
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

        if my_coords.is_empty() {
            return None;
        }

        // Find the closest enemy cell to any of my cells
        let (closest_my, closest_enemy, min_distance) = self.find_closest_pair(&my_coords, &enemy_coords);
        
        // Calculate the direction vector from my closest cell to enemy's closest cell
        let target_direction = if !enemy_coords.is_empty() {
            (
                closest_enemy.0 as isize - closest_my.0 as isize,
                closest_enemy.1 as isize - closest_my.1 as isize,
            )
        } else {
            // No enemy visible, head toward center
            let center = (board.rows / 2, board.cols / 2);
            let my_center = self.calculate_centroid(&my_coords);
            (
                center.0 as isize - my_center.0 as isize,
                center.1 as isize - my_center.1 as isize,
            )
        };

        // Find the frontier cells (my cells that can have pieces placed adjacent to them)
        let frontier = self.find_frontier(&my_coords, board);

        let mut best_pos: Option<(usize, usize)> = None;
        let mut best_score: i64 = i64::MIN;

        // Search entire board for valid placements
        let max_y = board.rows.saturating_sub(piece.height).saturating_add(1);
        let max_x = board.cols.saturating_sub(piece.width).saturating_add(1);
        
        for top_y in 0..max_y {
            for left_x in 0..max_x {
                if !self.is_valid_placement(board, piece, top_y, left_x) {
                    continue;
                }

                let score = self.score_placement(
                    board, piece, top_y, left_x,
                    &enemy_coords, &frontier,
                    target_direction, min_distance, closest_enemy
                );

                if score > best_score {
                    best_score = score;
                    best_pos = Some((top_y, left_x));
                }
            }
        }

        best_pos
    }

    fn find_closest_pair(
        &self,
        my_coords: &[(usize, usize)],
        enemy_coords: &[(usize, usize)],
    ) -> ((usize, usize), (usize, usize), usize) {
        if enemy_coords.is_empty() || my_coords.is_empty() {
            let my_first = my_coords.first().copied().unwrap_or((0, 0));
            return (my_first, (0, 0), usize::MAX);
        }

        let mut best_my = my_coords[0];
        let mut best_enemy = enemy_coords[0];
        let mut best_dist = usize::MAX;

        for &(my, mx) in my_coords {
            for &(ey, ex) in enemy_coords {
                let dist = (my as isize - ey as isize).unsigned_abs()
                    + (mx as isize - ex as isize).unsigned_abs();
                if dist < best_dist {
                    best_dist = dist;
                    best_my = (my, mx);
                    best_enemy = (ey, ex);
                }
            }
        }

        (best_my, best_enemy, best_dist)
    }

    fn calculate_centroid(&self, coords: &[(usize, usize)]) -> (usize, usize) {
        if coords.is_empty() {
            return (0, 0);
        }
        let sum_y: usize = coords.iter().map(|(y, _)| *y).sum();
        let sum_x: usize = coords.iter().map(|(_, x)| *x).sum();
        (sum_y / coords.len(), sum_x / coords.len())
    }

    fn find_frontier(&self, my_coords: &[(usize, usize)], board: &Board) -> Vec<(usize, usize)> {
        let mut frontier = Vec::new();
        const DIRS: &[(isize, isize)] = &[(1, 0), (-1, 0), (0, 1), (0, -1)];
        
        for &(y, x) in my_coords {
            for &(dy, dx) in DIRS {
                let ny = y as isize + dy;
                let nx = x as isize + dx;
                
                if ny >= 0 && nx >= 0 
                    && (ny as usize) < board.rows 
                    && (nx as usize) < board.cols
                    && board.cells[ny as usize][nx as usize] == Owner::Empty 
                {
                    frontier.push((y, x));
                    break;
                }
            }
        }
        frontier
    }

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
                Owner::Opponent => return false,
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

    fn score_placement(
        &self,
        board: &Board,
        piece: &Piece,
        top_y: usize,
        left_x: usize,
        enemy_coords: &[(usize, usize)],
        frontier: &[(usize, usize)],
        target_direction: (isize, isize),
        current_min_distance: usize,
        closest_enemy: (usize, usize),
    ) -> i64 {
        let rows = board.rows;
        let cols = board.cols;
        
        // Calculate where this placement puts us
        let mut piece_cells: Vec<(usize, usize)> = Vec::new();
        let mut new_territory: i64 = 0;
        let mut adjacent_to_enemy: i64 = 0;
        
        const DIRS: &[(isize, isize)] = &[(1, 0), (-1, 0), (0, 1), (0, -1)];

        for &(dy, dx) in &piece.cells {
            let ay = top_y + dy;
            let ax = left_x + dx;
            piece_cells.push((ay, ax));

            if board.cells[ay][ax] == Owner::Empty {
                new_territory += 1;
            }

            // Check for enemy adjacency
            for &(dyy, dxx) in DIRS {
                let ny = ay as isize + dyy;
                let nx = ax as isize + dxx;
                
                if ny >= 0 && nx >= 0 && (ny as usize) < rows && (nx as usize) < cols {
                    if board.cells[ny as usize][nx as usize] == Owner::Opponent {
                        adjacent_to_enemy += 1;
                    }
                }
            }
        }

        // Calculate the "most forward" point of this placement
        let mut best_advance: i64 = i64::MIN;
        let mut min_dist_to_enemy: usize = usize::MAX;
        
        for &(py, px) in &piece_cells {
            // How much does this cell advance toward target?
            // Use dot product with normalized direction
            let advance = if target_direction.0 != 0 || target_direction.1 != 0 {
                let norm = ((target_direction.0 * target_direction.0 + target_direction.1 * target_direction.1) as f64).sqrt();
                if norm > 0.0 {
                    // Project movement onto target direction
                    let move_y = py as isize - frontier.first().map(|f| f.0 as isize).unwrap_or(0);
                    let move_x = px as isize - frontier.first().map(|f| f.1 as isize).unwrap_or(0);
                    ((move_y * target_direction.0 + move_x * target_direction.1) as f64 / norm) as i64
                } else {
                    0
                }
            } else {
                0
            };
            
            if advance > best_advance {
                best_advance = advance;
            }

            // Distance to closest enemy
            for &(ey, ex) in enemy_coords {
                let d = (py as isize - ey as isize).unsigned_abs()
                    + (px as isize - ex as isize).unsigned_abs();
                if d < min_dist_to_enemy {
                    min_dist_to_enemy = d;
                }
            }
        }

        // Distance to the closest enemy cell we identified
        let dist_to_target = {
            let (ty, tx) = closest_enemy;
            let mut min_d = usize::MAX;
            for &(py, px) in &piece_cells {
                let d = (py as isize - ty as isize).unsigned_abs()
                    + (px as isize - tx as isize).unsigned_abs();
                if d < min_d {
                    min_d = d;
                }
            }
            min_d
        };

        // SCORING STRATEGY:
        // 1. If far from enemy (distance > 5): RUSH - minimize distance
        // 2. If close to enemy (distance <= 5): BLOCK - stay adjacent, expand around them
        
        if current_min_distance > 5 {
            // RUSH MODE: Get to enemy ASAP
            // Heavily reward reducing distance
            let distance_reduction = current_min_distance as i64 - min_dist_to_enemy as i64;
            let closeness_score = 1000000 / (min_dist_to_enemy as i64 + 1);
            
            closeness_score * 100           // Getting close is everything
            + distance_reduction * 50000    // Reward reducing distance
            + best_advance * 1000           // Reward advancing toward target
            + new_territory * 10            // Territory is almost irrelevant
            + adjacent_to_enemy * 100000    // If we can touch enemy, amazing!
        } else {
            // BLOCK MODE: We're close - now surround and contain
            let closeness_score = 100000 / (min_dist_to_enemy as i64 + 1);
            
            adjacent_to_enemy * 50000       // Stay glued to enemy
            + closeness_score * 50          // Stay close
            + new_territory * 2000          // Now territory matters
            + best_advance * 500            // Still advance when possible
            - (dist_to_target as i64) * 100 // Don't drift away from target
        }
    }
}
