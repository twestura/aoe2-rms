//! Lexes a RMS file into tokens.

use std::{iter::Peekable, path::Path, str::Chars};

/// Information for a token.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TokenInfo {
    line_number: usize,
    start_column: usize,
    end_column: usize,
    characters: String,
}

/// TODO
pub enum Token {
    Whitespace(TokenInfo),
    Text(TokenInfo),
}

impl Token {
    /// Returns a reference to the information associated with `self`.
    pub fn get_info(&self) -> &TokenInfo {
        match self {
            Self::Whitespace(t) => t,
            Self::Text(t) => t,
        }
    }
}
/// TODO
pub struct TokenizedFile {
    tokens: Vec<Token>,
}

/// Consumes and returns one token, text or whitespace, from `chars`.
/// Requires that `chars` contains no newlines.
/// If `chars` is empty, returns `None`. Otherwise returns `Some(token)`
/// while consuming the token from `chars`.
///
/// `line_number` is the 1-indexed number of the line at which the token is consumed.
/// `start_column` is the 1-indexed number of the column of the token's first character.
fn lex_one_token(
    line_number: usize,
    start_column: usize,
    chars: &mut Peekable<Chars>,
) -> Option<Token> {
    debug_assert!(line_number > 0);
    debug_assert!(start_column > 0);
    let mut characters = String::new();
    let mut num_chars = 0;
    let is_whitespace = chars.peek()?.is_whitespace();
    while let Some(&c) = chars.peek() {
        debug_assert!(c != '\n');
        if is_whitespace ^ c.is_whitespace() {
            break;
        }
        characters.push(c);
        num_chars += 1;
        chars.next();
    }
    let token_info = TokenInfo {
        line_number,
        start_column,
        end_column: start_column + num_chars - 1,
        characters,
    };
    Some(if is_whitespace {
        Token::Whitespace(token_info)
    } else {
        Token::Text(token_info)
    })
}

/// TODO
pub fn tokenize(path: &Path) -> std::io::Result<TokenizedFile> {
    // TODO buffered reader rather than read to string, analyze performance
    let s = std::fs::read_to_string(path)?;
    let mut tokens = vec![];
    for (i, line) in s.lines().enumerate() {
        let mut start_column = 1;
        let mut chars = line.chars().peekable();
        while let Some(token) = lex_one_token(i + 1, start_column, &mut chars) {
            start_column = token.get_info().end_column + 1;
            tokens.push(token);
        }
    }

    Ok(TokenizedFile { tokens })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lexing one token from an empty iterator produces `None`.
    #[test]
    fn lex_one_token_empty() {
        let s = String::new();
        let mut chars = s.chars().peekable();
        let result = lex_one_token(1, 1, &mut chars);
        assert!(result.is_none());
    }

    /// Lexing one token from a whitespace only string is nonempty.
    #[test]
    fn lex_one_token_nonempty_whitespace() {
        let s = String::from("        \t\t  ");
        let mut chars = s.chars().peekable();
        let result = lex_one_token(1, 1, &mut chars);
        assert!(result.is_some());
    }

    #[test]
    /// Lexing one token from a text only string is nonempty.
    fn lex_one_token_nonempty_text() {
        let s = String::from("base_terrain");
        let mut chars = s.chars().peekable();
        let result = lex_one_token(1, 1, &mut chars);
        assert!(result.is_some());
    }

    #[test]
    /// Lexing one token from a generic string with whitespace and text is nonempty.
    fn lex_one_token_nonempty_generic() {
        let s = String::from("  base_terrain GRASS\n  land_percent 50\n  base_size 7\n");
        let mut chars = s.chars().peekable();
        let result = lex_one_token(1, 1, &mut chars);
        assert!(result.is_some());
    }

    // TODO write more tests
}
