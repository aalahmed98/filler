// src/main.rs

mod parser;
mod board;
mod piece;
mod game;

use std::io::{self, BufRead, Write};

use crate::parser::parse_player_number;
use crate::board::Board;
use crate::piece::Piece;
use crate::game::Game;

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    // 1) Detect which player we are
    let my_player = loop {
        match lines.next() {
            Some(Ok(line)) => {
                if let Some(num) = parse_player_number(&line) {
                    break num;
                }
                // Ignore unrelated lines until we find the exec line
            }
            _ => {
                // No input
                return;
            }
        }
    };

    //unused??
    let game = Game::new(my_player); 
    
    // 2) Main game loop: each iteration = one turn
    'game_loop: loop {
        // Collect Anfield block
        let mut anfield_lines: Vec<String> = Vec::new();

        // Find "Anfield" header
        let header = loop {
            match lines.next() {
                Some(Ok(line)) => {
                    if line.trim_start().starts_with("Anfield") {
                        break line;
                    }
                    // Ignore other lines until we see Anfield
                }
                _ => {
                    // No more data, game over
                    return;
                }
            }
        };

        anfield_lines.push(header.clone());

        // Read until we see "Piece" header
        let piece_header: String;
        loop {
            match lines.next() {
                Some(Ok(line)) => {
                    if line.trim_start().starts_with("Piece") {
                        piece_header = line;
                        break;
                    } else {
                        anfield_lines.push(line);
                    }
                }
                _ => {
                    // EOF before piece, stop
                    return;
                }
            }
        }

        // Parse the board from the collected lines
        let board = match Board::from_anfield_lines(&anfield_lines, my_player) {
            Some(b) => b,
            None => break 'game_loop,
        };

        // Collect piece block: header + height lines
        let mut piece_lines: Vec<String> = Vec::new();
        piece_lines.push(piece_header.clone());

        let height = {
            let trimmed = piece_header.trim();
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 3 {
                parts[2]
                    .trim_end_matches(':')
                    .parse::<usize>()
                    .unwrap_or(0)
            } else {
                0
            }
        };
        
        for _ in 0..height {
            match lines.next() {
                Some(Ok(line)) => piece_lines.push(line),
                _ => {
                    // Incomplete piece, stop the game
                    break 'game_loop;
                }
            }
        }
        

        let piece = match Piece::from_piece_lines(&piece_lines) {
            Some(p) => p,
            None => break 'game_loop,
        };

        // Ask the strategy for the best move
        let (out_y, out_x) = match game.choose_best_move(&board, &piece) {
            Some((y, x)) => (y, x),
            None => (0usize, 0usize), // fallback if no valid placement
        };

        println!("{} {}", out_y, out_x);
        let _ = io::stdout().flush();
    }
}
