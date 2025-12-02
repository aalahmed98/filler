// src/parser.rs

pub fn parse_player_number(line: &str) -> Option<u8> {
    let trimmed = line.trim();

    // Expected format: `$$$ exec p1 : ...`
    if !trimmed.starts_with("$$$ exec p") {
        return None;
    }

    // Strip the prefix
    let prefix = "$$$ exec p";
    let after = &trimmed[prefix.len()..];

    // Everything until ':' should be the player number
    if let Some(colon_pos) = after.find(':') {
        let num_str = after[..colon_pos].trim();
        if let Ok(num) = num_str.parse::<u8>() {
            if num == 1 || num == 2 {
                return Some(num);
            }
        }
    }

    None
}
