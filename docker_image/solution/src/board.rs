// src/board.rs

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Owner {
    Empty,
    Me,
    Opponent,
}

pub struct Board {
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<Vec<Owner>>,
}

impl Board {
    /// Build a board from the "Anfield" block lines.
    ///
    /// The block looks something like:
    /// Anfield <cols> <rows>:
    ///     012345...
    /// 000 ....@...
    /// 001 ...$....
    ///
    /// Player 1 uses '@' / 'a', player 2 uses '$' / 's'.
    pub fn from_anfield_lines(lines: &[String], my_player: u8) -> Option<Self> {
        if lines.is_empty() {
            return None;
        }

        let mut grid: Vec<Vec<Owner>> = Vec::new();
        let mut seen_header = false;

        for line in lines {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            // Skip header line but mark that we've seen it
            if trimmed.starts_with("Anfield") {
                seen_header = true;
                continue;
            }

            if !seen_header {
                // Ignore anything before the "Anfield" header
                continue;
            }

            // Skip lines that are just row/column indices
            if trimmed
                .chars()
                .all(|c| c.is_ascii_digit() || c.is_whitespace())
            {
                continue;
            }

            // Strip leading row indices and whitespace
            let row_str: String = line
                .chars()
                .skip_while(|c| c.is_ascii_digit() || c.is_whitespace())
                .collect();

            if row_str.is_empty() {
                continue;
            }

            let mut row: Vec<Owner> = Vec::with_capacity(row_str.len());

            // IMPORTANT FIX: skip spaces between cells
            for ch in row_str.chars() {
                if ch == ' ' {
                    continue;
                }
                let owner = classify_char(ch, my_player);
                row.push(owner);
            }

            if !row.is_empty() {
                grid.push(row);
            }
        }

        if grid.is_empty() {
            return None;
        }

        let rows = grid.len();
        let cols = grid[0].len();

        Some(Board { rows, cols, cells: grid })
    }
}

// IMPORTANT FIX: treat unknown characters as Empty, not Opponent
fn classify_char(c: char, my_player: u8) -> Owner {
    match c {
        '.' => Owner::Empty,

        '@' | 'a' => {
            if my_player == 1 {
                Owner::Me
            } else {
                Owner::Opponent
            }
        }

        '$' | 's' => {
            if my_player == 2 {
                Owner::Me
            } else {
                Owner::Opponent
            }
        }

        _ => Owner::Empty,
    }
}
