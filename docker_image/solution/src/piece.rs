// src/piece.rs

pub struct Piece {
    pub width: usize,
    pub height: usize,
    /// Coordinates of filled cells relative to the top-left of the piece.
    pub cells: Vec<(usize, usize)>,
}

impl Piece {
    /// Build a piece from the "Piece" block lines.
    ///
    /// Example:
    /// Piece 3 5:
    /// ..*..
    /// .***.
    /// ..*..
    ///
    /// We treat '*', 'O', 'o' as filled.
    /// IMPORTANT: format is "Piece <width> <height>:"
    pub fn from_piece_lines(lines: &[String]) -> Option<Self> {
        if lines.is_empty() {
            return None;
        }

        // Find the header line
        let header_index = lines
            .iter()
            .position(|l| l.trim_start().starts_with("Piece"))?;

        let header = lines[header_index].trim();
        let parts: Vec<&str> = header.split_whitespace().collect();

        if parts.len() < 3 {
            return None;
        }

        // Correct interpretation:
        // "Piece <width> <height>:"
        let expected_width: usize = parts[1].parse().ok()?;
        let expected_height: usize = parts[2]
            .trim_end_matches(':')
            .parse()
            .ok()?;

        // Collect piece pattern lines after the header
        let mut pattern: Vec<String> = Vec::new();
        for line in lines.iter().skip(header_index + 1) {
            let t = line.trim_end().to_string();
            if t.is_empty() {
                continue;
            }
            pattern.push(t);
            if pattern.len() == expected_height {
                break;
            }
        }

        if pattern.is_empty() {
            return None;
        }

        let height = pattern.len();
        let mut width = expected_width;
        let mut filled_cells: Vec<(usize, usize)> = Vec::new();

        for (y, row_str) in pattern.iter().enumerate() {
            let row_len = row_str.chars().count();
            if row_len > width {
                width = row_len;
            }

            for (x, ch) in row_str.chars().enumerate() {
                if ch == '*' || ch == 'O' || ch == 'o' {
                    filled_cells.push((y, x));
                }
            }
        }

        if filled_cells.is_empty() {
            // No useful piece
            return None;
        }

        Some(Piece {
            width,
            height,
            cells: filled_cells,
        })
    }
}
