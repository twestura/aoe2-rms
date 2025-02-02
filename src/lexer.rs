//! Lexes a RMS file into tokens.

use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    iter::Peekable,
    path::Path,
    str::Chars,
};

/// Information for a lexeme.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct LexemeInfo {
    /// The 1-indexed line number of the lexeme.
    line_number: usize,
    /// The 1-indexed column number of the first character of the lexeme.
    start_column: usize,
    /// The 1-indexed column number of the final character of hte lexeme.
    end_column: usize,
    /// The sequence of characters comprising the lexeme.
    characters: String,
}

impl LexemeInfo {
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

/// A lexeme parsed from an RMS file.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Lexeme {
    /// A line break: `\r\n` or `\n`.
    LineBreak(LexemeInfo),
    /// A consecutive sequence of whitespace characters that is not a linebreak.
    Whitespace(LexemeInfo),
    /// A lexeme of non-whitespace characters.
    Text(LexemeInfo),
}

impl Lexeme {
    /// Returns a reference to the information associated with `self`.
    pub fn get_info(&self) -> &LexemeInfo {
        match self {
            Self::LineBreak(t) => t,
            Self::Whitespace(t) => t,
            Self::Text(t) => t,
        }
    }
}
/// A sequence of lexemes comprising a file.
/// Using the information stored in each lexeme, the file may be reconstructed
/// exactly as it was before it was parsed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LexemeFile {
    lexemes: Vec<Lexeme>,
}

impl LexemeFile {
    /// Writes to the file at `path`, overwriting the file if it exists.
    /// Returns an io error if the writing fails.
    /// Note that an existing file may still be overwritten even if writing fails.
    pub fn write_to_path(&self, path: &Path) -> std::io::Result<()> {
        let mut f = File::create(path)?;
        for lexeme in self.lexemes.iter() {
            write!(f, "{}", lexeme.get_info().characters)?;
        }
        Ok(())
    }

    /// Returns a reference to the vector of lexemes in this file.
    pub fn lexemes(&self) -> &Vec<Lexeme> {
        &self.lexemes
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

/// Consumes and returns one lexeme, text or whitespace, from `chars`.
/// Requires that `chars` contains no line breaks, that is, no `\n` characters.
/// If `chars` is empty, returns `None`. Otherwise returns `Some(lexeme)`
/// while consuming the lexeme from `chars`.
///
/// `line_number` is the 1-indexed number of the line at which the lexeme is consumed.
/// `start_column` is the 1-indexed number of the column of the lexeme's first character.
fn lex_one_lexeme(
    line_number: usize,
    start_column: usize,
    chars: &mut Peekable<Chars>,
) -> Option<Lexeme> {
    debug_assert!(line_number > 0);
    debug_assert!(start_column > 0);
    let mut characters = String::new();
    let mut num_chars = 0;
    let whitespace_lexeme = is_whitespace(*chars.peek()?);
    while let Some(&c) = chars.peek() {
        debug_assert!(c != '\n', "The line has a line feed char.");
        // Stop when detecting a different type of character.
        if whitespace_lexeme ^ is_whitespace(c) {
            break;
        }
        characters.push(c);
        num_chars += 1;
        chars.next();
    }
    let lexeme_info = LexemeInfo {
        line_number,
        start_column,
        end_column: start_column + num_chars - 1,
        characters,
    };
    Some(if whitespace_lexeme {
        Lexeme::Whitespace(lexeme_info)
    } else {
        Lexeme::Text(lexeme_info)
    })
}

/// Returns a pair `(line_content, Some(line_break_info))`.
/// If `line` ends with a line break sequence, either `\r\n`, or `\n`,
/// then that sequence is extracted into the information for a `LineBreak` lexeme,
/// and the returned `line_content` references the `line` without the ending break.
///
/// Requires that, if `line` contains a linebreak, then the break is at the end.
/// Requires `line_number >= 1`.
fn extract_line_break(line: &str, line_number: usize) -> (&str, Option<LexemeInfo>) {
    debug_assert!(line_number >= 1);
    // The debug assertions enforce the precondition of containing the linebreak
    // only at the end. The `line`s are collected from the `lines` of a buffered reader,
    // which should not produce "internal" line breaks.
    if line.ends_with("\r\n") {
        debug_assert!(line.chars().filter(|c| *c == '\n').count() == 1);
        // Note `col` is 0-indexed, whereas the start and end columns are 1-indexed.
        let col = line.len() - 2;
        (
            &line[..col],
            Some(LexemeInfo {
                line_number,
                start_column: col + 1,
                end_column: col + 2,
                characters: String::from("\r\n"),
            }),
        )
    } else if line.ends_with('\n') {
        debug_assert!(line.chars().filter(|c| *c == '\n').count() == 1);
        // Note `col` is 0-indexed, whereas the start and end columns are 1-indexed.
        let col = line.len() - 1;
        (
            &line[..col],
            Some(LexemeInfo {
                line_number,
                start_column: col + 1,
                end_column: col + 1,
                characters: String::from("\n"),
            }),
        )
    } else {
        debug_assert!(line.chars().filter(|c| *c == '\n').count() == 0);
        (line, None)
    }
}

/// Turns the rms script in the file located at `path` into a sequence of lexemes.
/// Returns the lexemes.
/// Returns an error if there is an io error in processing the file at `path`.
pub fn lex(path: &Path) -> std::io::Result<LexemeFile> {
    let f = File::open(path)?;
    let mut br = BufReader::new(f);
    let mut lexemes = vec![];
    let mut line_number = 1;
    let mut line = String::new();
    while br.read_line(&mut line)? > 0 {
        let (line_content, line_break) = extract_line_break(&line, line_number);
        let mut start_column = 1;
        let mut chars = line_content.chars().peekable();
        while let Some(lexeme) = lex_one_lexeme(line_number, start_column, &mut chars) {
            start_column = lexeme.get_info().end_column + 1;
            lexemes.push(lexeme);
        }
        if let Some(break_info) = line_break {
            lexemes.push(Lexeme::LineBreak(break_info));
        }
        line_number += 1;
        line.clear();
    }
    Ok(LexemeFile { lexemes })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lexing one lexeme from an empty iterator produces `None`.
    #[test]
    fn lex_one_lexeme_empty() {
        let s = String::new();
        let mut chars = s.chars().peekable();
        let result = lex_one_lexeme(1, 1, &mut chars);
        assert!(result.is_none());
    }

    /// Lexing one lexeme from a whitespace only string is nonempty.
    #[test]
    fn lex_one_lexeme_nonempty_whitespace() {
        let s = String::from("        \t\t  ");
        let mut chars = s.chars().peekable();
        let result = lex_one_lexeme(1, 1, &mut chars);
        assert!(result.is_some());
    }

    /// Lexing one lexeme from a text only string is nonempty.
    #[test]
    fn lex_one_lexeme_nonempty_text() {
        let s = String::from("base_terrain");
        let mut chars = s.chars().peekable();
        let result = lex_one_lexeme(1, 1, &mut chars);
        assert!(result.is_some());
    }

    /// Lexing one lexeme from a generic string with whitespace and text is nonempty.
    #[test]
    fn lex_one_lexeme_nonempty_generic() {
        let s = String::from("\tbase_terrain GRASS land_percent 50 base_size 7");
        let mut chars = s.chars().peekable();
        let result = lex_one_lexeme(1, 1, &mut chars);
        assert!(result.is_some());
    }

    #[test]
    /// Lexes all lexemes from one line of code with whitespace and text characters.
    fn lex_one_line() {
        let s = String::from("\tbase_terrain GRASS land_percent 50 base_size 7");
        let mut chars = s.chars().peekable();

        // First tab character.
        let result = lex_one_lexeme(1, 1, &mut chars).unwrap();
        let info = match result {
            Lexeme::Whitespace(info) => info,
            _ => panic!("Lexeme must be whitespace."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 1);
        assert_eq!(info.end_column, 1);
        assert_eq!(info.characters, "\t");

        // base_terrain lexeme
        let result = lex_one_lexeme(1, 2, &mut chars).unwrap();
        let info = match result {
            Lexeme::Text(info) => info,
            _ => panic!("Lexeme must be text."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 2);
        assert_eq!(info.end_column, 13);
        assert_eq!(info.characters, "base_terrain");

        // Space after base_terrain
        let result = lex_one_lexeme(1, 14, &mut chars).unwrap();
        let info = match result {
            Lexeme::Whitespace(info) => info,
            _ => panic!("Lexeme must be whitespace."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 14);
        assert_eq!(info.end_column, 14);
        assert_eq!(info.characters, " ");

        // GRASS lexeme
        let result = lex_one_lexeme(1, 15, &mut chars).unwrap();
        let info = match result {
            Lexeme::Text(info) => info,
            _ => panic!("Lexeme must be text."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 15);
        assert_eq!(info.end_column, 19);
        assert_eq!(info.characters, "GRASS");

        // Space after GRASS
        let result = lex_one_lexeme(1, 20, &mut chars).unwrap();
        let info = match result {
            Lexeme::Whitespace(info) => info,
            _ => panic!("Lexeme must be whitespace."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 20);
        assert_eq!(info.end_column, 20);
        assert_eq!(info.characters, " ");

        // land_percent lexeme
        let result = lex_one_lexeme(1, 21, &mut chars).unwrap();
        let info = match result {
            Lexeme::Text(info) => info,
            _ => panic!("Lexeme must be text."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 21);
        assert_eq!(info.end_column, 32);
        assert_eq!(info.characters, "land_percent");

        // Space after land_percent
        let result = lex_one_lexeme(1, 33, &mut chars).unwrap();
        let info = match result {
            Lexeme::Whitespace(info) => info,
            _ => panic!("Lexeme must be whitespace."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 33);
        assert_eq!(info.end_column, 33);
        assert_eq!(info.characters, " ");

        // 50 lexeme
        let result = lex_one_lexeme(1, 34, &mut chars).unwrap();
        let info = match result {
            Lexeme::Text(info) => info,
            _ => panic!("Lexeme must be text."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 34);
        assert_eq!(info.end_column, 35);
        assert_eq!(info.characters, "50");

        // Space after 50
        let result = lex_one_lexeme(1, 36, &mut chars).unwrap();
        let info = match result {
            Lexeme::Whitespace(info) => info,
            _ => panic!("Lexeme must be whitespace."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 36);
        assert_eq!(info.end_column, 36);
        assert_eq!(info.characters, " ");

        // base_size lexeme
        let result = lex_one_lexeme(1, 37, &mut chars).unwrap();
        let info = match result {
            Lexeme::Text(info) => info,
            _ => panic!("Lexeme must be text."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 37);
        assert_eq!(info.end_column, 45);
        assert_eq!(info.characters, "base_size");

        // Space after base_size
        let result = lex_one_lexeme(1, 46, &mut chars).unwrap();
        let info = match result {
            Lexeme::Whitespace(info) => info,
            _ => panic!("Lexeme must be whitespace."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 46);
        assert_eq!(info.end_column, 46);
        assert_eq!(info.characters, " ");

        // 7 lexeme
        let result = lex_one_lexeme(1, 47, &mut chars).unwrap();
        let info = match result {
            Lexeme::Text(info) => info,
            _ => panic!("Lexeme must be text."),
        };
        assert_eq!(info.line_number, 1);
        assert_eq!(info.start_column, 47);
        assert_eq!(info.end_column, 47);
        assert_eq!(info.characters, "7");

        let result = lex_one_lexeme(1, 48, &mut chars);
        assert!(result.is_none());
    }

    /// Continuing to lex after the end of a sequence returns `None`.
    #[test]
    fn lex_one_lexeme_multiple_none() {
        let s = String::from("GRASS");
        let mut chars = s.chars().peekable();
        assert!(lex_one_lexeme(1, 1, &mut chars).is_some());
        assert!(lex_one_lexeme(1, 5, &mut chars).is_none());
        assert!(lex_one_lexeme(1, 5, &mut chars).is_none());
        for _ in 0..10 {
            assert!(lex_one_lexeme(1, 5, &mut chars).is_none());
        }
    }

    /// Mixed whitespace characters (e.g. tabs and spaces) are included together.
    #[test]
    fn lex_one_lexeme_mixed_whitespace() {
        let s = String::from("  \t \t\t ");
        let mut chars = s.chars().peekable();
        let result = lex_one_lexeme(1, 1, &mut chars).unwrap();
        let info = match result {
            Lexeme::Whitespace(info) => info,
            _ => panic!("Lexeme must be text."),
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

    /// Tests that no line break is extracted from a string without an end break.
    #[test]
    fn extract_no_line_break() {
        let (content, info) = extract_line_break("base_terrain GRASS", 1);
        assert_eq!(content, "base_terrain GRASS");
        assert!(info.is_none());
    }

    /// Tests that a carriage return is not counted as a line break.
    #[test]
    fn extract_no_carriage_return_character() {
        let (content, info) = extract_line_break("base_terrain GRASS\r", 1);
        assert_eq!(content, "base_terrain GRASS\r");
        assert!(info.is_none());
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
