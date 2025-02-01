//! Lexes a RMS file into tokens.

use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    iter::Peekable,
    path::Path,
    str::Chars,
};

/// Information for a token.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TokenInfo {
    line_number: usize,
    start_column: usize,
    end_column: usize,
    characters: String,
}

impl TokenInfo {
    /// Returns this token's 1-indexed line number.
    pub fn line_number(&self) -> usize {
        self.line_number
    }

    /// Returns this token's 1-indexed start column.
    pub fn start_column(&self) -> usize {
        self.start_column
    }

    /// Returns this token's 1-indexed end column.
    pub fn end_column(&self) -> usize {
        self.end_column
    }

    /// Returns a reference to this token's characters.
    pub fn characters(&self) -> &str {
        &self.characters
    }
}

/// A token parsed from an RMS file.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Token {
    /// A line break (`/r/n`, `/n`, or `/r`).
    LineBreak(TokenInfo),
    /// A consecutive sequence of whitespace characters that is not a linebreak.
    Whitespace(TokenInfo),
    /// A token of non-whitespace characters.
    Text(TokenInfo),
}

impl Token {
    /// Returns a reference to the information associated with `self`.
    pub fn get_info(&self) -> &TokenInfo {
        match self {
            Self::LineBreak(t) => t,
            Self::Whitespace(t) => t,
            Self::Text(t) => t,
        }
    }
}
/// A sequence of tokens comprising a file.
/// Using the information stored in each token, the file may be reconstructed
/// exactly as it was before it was parsed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenizedFile {
    tokens: Vec<Token>,
}

impl TokenizedFile {
    /// Writes to the file at `path`, overwriting the file if it exists.
    /// Returns an io error if the writing fails.
    /// Note that an existing file may still be overwritten even if writing fails.
    pub fn write_to_path(&self, path: &Path) -> std::io::Result<()> {
        let mut f = File::create(path)?;
        for token in self.tokens.iter() {
            write!(f, "{}", token.get_info().characters)?;
        }
        Ok(())
    }

    /// Returns a reference to the vector of tokens in this file.
    pub fn tokens(&self) -> &Vec<Token> {
        &self.tokens
    }
}

/// Returns `true` if `c` is considered a whitespace character in RMS scripts.
/// Returns `false` if not.
///
/// Note that non-ascii unicode whitespace characters are not considered whitespace
/// in map scripts.
pub fn is_whitespace(c: char) -> bool {
    c.is_ascii_whitespace()
}

/// Consumes and returns one token, text or whitespace, from `chars`.
/// Requires that `chars` contains no line breaks, that is, no `\r` and no `\n` characters.
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
    let whitespace_token = is_whitespace(*chars.peek()?);
    while let Some(&c) = chars.peek() {
        debug_assert!(c != '\r' && c != '\n', "The line has a line feed char.");
        // Stop when detecting a different type of character.
        if whitespace_token ^ is_whitespace(c) {
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
    Some(if whitespace_token {
        Token::Whitespace(token_info)
    } else {
        Token::Text(token_info)
    })
}

/// Returns a pair `(line_content, Some(line_break_info))`.
/// If `line` ends with a line break sequence, either `\r\n`, `\r`, or `\n`,
/// then that sequence is extracted into the information for a `LineBreak` token,
/// and the returned `line_content` references the `line` without the ending break.
///
/// Requires that, if `line` contains a linebreak, then the break is at the end.
/// Requires `line_number >= 1`.
fn extract_line_break(line: &str, line_number: usize) -> (&str, Option<TokenInfo>) {
    debug_assert!(line_number >= 1);
    // The debug assertions enforce the precondition of containing the linebreak
    // only at the end. The `line`s are collected from the `lines` of a buffered reader,
    // which should not produce "internal" line breaks.
    if line.ends_with("\r\n") {
        debug_assert!(line.chars().filter(|c| *c == '\r' || *c == '\n').count() == 2);
        // Note `col` is 0-indexed, whereas the start and end columns are 1-indexed.
        let col = line.len() - 2;
        (
            &line[..col],
            Some(TokenInfo {
                line_number,
                start_column: col + 1,
                end_column: col + 2,
                characters: String::from("\r\n"),
            }),
        )
    } else if line.ends_with('\r') {
        debug_assert!(line.chars().filter(|c| *c == '\r' || *c == '\n').count() == 1);
        // Note `col` is 0-indexed, whereas the start and end columns are 1-indexed.
        let col = line.len() - 1;
        (
            &line[..col],
            Some(TokenInfo {
                line_number,
                start_column: col + 1,
                end_column: col + 1,
                characters: String::from("\r"),
            }),
        )
    } else if line.ends_with('\n') {
        debug_assert!(line.chars().filter(|c| *c == '\r' || *c == '\n').count() == 1);
        // Note `col` is 0-indexed, whereas the start and end columns are 1-indexed.
        let col = line.len() - 1;
        (
            &line[..col],
            Some(TokenInfo {
                line_number,
                start_column: col + 1,
                end_column: col + 1,
                characters: String::from("\n"),
            }),
        )
    } else {
        debug_assert!(line.chars().filter(|c| *c == '\r' || *c == '\n').count() == 0);
        (line, None)
    }
}

/// Tokenizes the rms script in the file located at `path`.
/// Returns the tokenized file.
/// Returns an error if there is an io error in processing the file at `path`.
pub fn tokenize(path: &Path) -> std::io::Result<TokenizedFile> {
    let f = File::open(path)?;
    let mut br = BufReader::new(f);
    let mut tokens = vec![];
    let mut line_number = 1;
    let mut line = String::new();
    while br.read_line(&mut line)? > 0 {
        let (line_content, line_break) = extract_line_break(&line, line_number);
        let mut start_column = 1;
        let mut chars = line_content.chars().peekable();
        while let Some(token) = lex_one_token(line_number, start_column, &mut chars) {
            start_column = token.get_info().end_column + 1;
            tokens.push(token);
        }
        if let Some(break_info) = line_break {
            tokens.push(Token::LineBreak(break_info));
        }
        line_number += 1;
        line.clear();
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

    /// Lexing one token from a text only string is nonempty.
    #[test]
    fn lex_one_token_nonempty_text() {
        let s = String::from("base_terrain");
        let mut chars = s.chars().peekable();
        let result = lex_one_token(1, 1, &mut chars);
        assert!(result.is_some());
    }

    /// Lexing one token from a generic string with whitespace and text is nonempty.
    #[test]
    fn lex_one_token_nonempty_generic() {
        let s = String::from("\tbase_terrain GRASS land_percent 50 base_size 7");
        let mut chars = s.chars().peekable();
        let result = lex_one_token(1, 1, &mut chars);
        assert!(result.is_some());
    }

    #[test]
    /// Lexes all tokens from one line of code with whitespace and text characters.
    fn lex_one_line() {
        let s = String::from("\tbase_terrain GRASS land_percent 50 base_size 7");
        let mut chars = s.chars().peekable();

        // First tab character.
        let result = lex_one_token(1, 1, &mut chars).unwrap();
        let info = match result {
            Token::Whitespace(info) => info,
            _ => panic!("Token must be whitespace."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 1);
        assert_eq!(info.end_column, 1);
        assert_eq!(info.characters, "\t");

        // base_terrain token
        let result = lex_one_token(1, 2, &mut chars).unwrap();
        let info = match result {
            Token::Text(info) => info,
            _ => panic!("Token must be text."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 2);
        assert_eq!(info.end_column, 13);
        assert_eq!(info.characters, "base_terrain");

        // Space after base_terrain
        let result = lex_one_token(1, 14, &mut chars).unwrap();
        let info = match result {
            Token::Whitespace(info) => info,
            _ => panic!("Token must be whitespace."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 14);
        assert_eq!(info.end_column, 14);
        assert_eq!(info.characters, " ");

        // GRASS token
        let result = lex_one_token(1, 15, &mut chars).unwrap();
        let info = match result {
            Token::Text(info) => info,
            _ => panic!("Token must be text."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 15);
        assert_eq!(info.end_column, 19);
        assert_eq!(info.characters, "GRASS");

        // Space after GRASS
        let result = lex_one_token(1, 20, &mut chars).unwrap();
        let info = match result {
            Token::Whitespace(info) => info,
            _ => panic!("Token must be whitespace."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 20);
        assert_eq!(info.end_column, 20);
        assert_eq!(info.characters, " ");

        // land_percent token
        let result = lex_one_token(1, 21, &mut chars).unwrap();
        let info = match result {
            Token::Text(info) => info,
            _ => panic!("Token must be text."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 21);
        assert_eq!(info.end_column, 32);
        assert_eq!(info.characters, "land_percent");

        // Space after land_percent
        let result = lex_one_token(1, 33, &mut chars).unwrap();
        let info = match result {
            Token::Whitespace(info) => info,
            _ => panic!("Token must be whitespace."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 33);
        assert_eq!(info.end_column, 33);
        assert_eq!(info.characters, " ");

        // 50 token
        let result = lex_one_token(1, 34, &mut chars).unwrap();
        let info = match result {
            Token::Text(info) => info,
            _ => panic!("Token must be text."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 34);
        assert_eq!(info.end_column, 35);
        assert_eq!(info.characters, "50");

        // Space after 50
        let result = lex_one_token(1, 36, &mut chars).unwrap();
        let info = match result {
            Token::Whitespace(info) => info,
            _ => panic!("Token must be whitespace."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 36);
        assert_eq!(info.end_column, 36);
        assert_eq!(info.characters, " ");

        // base_size token
        let result = lex_one_token(1, 37, &mut chars).unwrap();
        let info = match result {
            Token::Text(info) => info,
            _ => panic!("Token must be text."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 37);
        assert_eq!(info.end_column, 45);
        assert_eq!(info.characters, "base_size");

        // Space after base_size
        let result = lex_one_token(1, 46, &mut chars).unwrap();
        let info = match result {
            Token::Whitespace(info) => info,
            _ => panic!("Token must be whitespace."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 46);
        assert_eq!(info.end_column, 46);
        assert_eq!(info.characters, " ");

        // 7 token
        let result = lex_one_token(1, 47, &mut chars).unwrap();
        let info = match result {
            Token::Text(info) => info,
            _ => panic!("Token must be text."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 47);
        assert_eq!(info.end_column, 47);
        assert_eq!(info.characters, "7");

        let result = lex_one_token(1, 48, &mut chars);
        assert!(result.is_none());
    }

    /// Continuing to lex after the end of a sequence returns `None`.
    #[test]
    fn lex_one_token_multiple_none() {
        let s = String::from("GRASS");
        let mut chars = s.chars().peekable();
        assert!(lex_one_token(1, 1, &mut chars).is_some());
        assert!(lex_one_token(1, 5, &mut chars).is_none());
        assert!(lex_one_token(1, 5, &mut chars).is_none());
        for _ in 0..10 {
            assert!(lex_one_token(1, 5, &mut chars).is_none());
        }
    }

    /// Mixed whitespace characters (e.g. tabs and spaces) are included together.
    #[test]
    fn lex_one_token_mixed_whitespace() {
        let s = String::from("  \t \t\t ");
        let mut chars = s.chars().peekable();
        let result = lex_one_token(1, 1, &mut chars).unwrap();
        let info = match result {
            Token::Whitespace(info) => info,
            _ => panic!("Token must be text."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 1);
        assert_eq!(info.end_column, 7);
        assert_eq!(info.characters, "  \t \t\t ");
    }

    /// Tests that no line break is extracted from an empty string.
    #[test]
    fn extract_line_break_empty() {
        let (content, info) = extract_line_break("", 1);
        assert_eq!(content, "");
        assert!(info.is_none());
    }

    /// Tests that no line breka is extracted from a string without an end break.
    #[test]
    fn extract_no_line_break() {
        let (content, info) = extract_line_break("base_terrain GRASS", 1);
        assert_eq!(content, "base_terrain GRASS");
        assert!(info.is_none());
    }

    /// Tests extracting a carriage return.
    #[test]
    fn extract_carriage_return_character() {
        let (content, info) = extract_line_break("base_terrain GRASS\r", 1);
        assert_eq!(content, "base_terrain GRASS");
        let info = info.unwrap();
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 19);
        assert_eq!(info.end_column, 19);
        assert_eq!(info.characters, "\r");
    }

    /// Tests extracting a line feed.
    #[test]
    fn extract_line_feed_character() {
        let (content, info) = extract_line_break("base_terrain GRASS\n", 1);
        assert_eq!(content, "base_terrain GRASS");
        let info = info.unwrap();
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 19);
        assert_eq!(info.end_column, 19);
        assert_eq!(info.characters, "\n");
    }

    /// Tests extracting a `\r\n` sequence.
    #[test]
    fn extract_line_break_sequence() {
        let (content, info) = extract_line_break("base_terrain GRASS\r\n", 1);
        assert_eq!(content, "base_terrain GRASS");
        let info = info.unwrap();
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 19);
        assert_eq!(info.end_column, 20);
        assert_eq!(info.characters, "\r\n");
    }
}
