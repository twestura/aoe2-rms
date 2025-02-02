//! Tools for writing a parsed RMS file to a debugging HTML file.

use std::{fs::File, io::Write, path::Path};

use crate::{
    annotater::{AnnotatedFile, AnnotatedToken},
    lexer::{Lexeme, LexemeFile},
};

/// The `<head>` section of the html file.
const HTML_HEAD: &str = r#"  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <link rel="stylesheet" href="style.css" />
    <title>Code</title>
  </head>"#;

/// Replaces characters in `s` so that they show up in html.
///
/// Performs the following replacements:
///
/// - `<` to `&le;`
/// - `>` to `&ge;`
fn transform_text_to_html(s: &str) -> String {
    // TODO more replacements
    s.replace('<', "&lt;").replace('>', "&gt;")
}

/// Writes a debug file using just the lexemes, without tokenization or annotation.
/// `lexemes` is the map script's sequence of lexemes.
/// `output` is the path to which the output file is written. If a file already exists, it
/// is overwritten.
/// Returns an IO error if there is an error writing to the `output` file.
pub fn write_debug_file(lexemes: &LexemeFile, output: &Path) -> std::io::Result<()> {
    let mut f = File::create(output)?;
    writeln!(f, "<!DOCTYPE html>")?;
    writeln!(f, "<html lang=\"en\">")?;
    writeln!(f, "{HTML_HEAD}")?;
    writeln!(f, "  <body>")?;
    writeln!(f, "    <ol>")?;
    let mut line_in_progress = false;
    for token in lexemes.lexemes() {
        if !line_in_progress {
            writeln!(f, "      <li>")?;
            write!(f, "        <pre><code>")?;
            line_in_progress = true;
        }
        match token {
            Lexeme::LineBreak(_token_info) => {
                write!(f, "</code></pre>\n")?;
                writeln!(f, "      </li>")?;
                line_in_progress = false;
            }
            Lexeme::Whitespace(token_info) => {
                write!(f, "{}", token_info.characters())?;
            }
            Lexeme::Text(token_info) => {
                let html = transform_text_to_html(token_info.characters());
                let start = token_info.start_column();
                let end = token_info.end_column();
                let range_display = if start == end {
                    format!("{start}")
                } else {
                    format!("{start}&ndash;{end}")
                };
                let card = format!("<div>{range_display}</div>",);
                write!(
                    f,
                    "<span class=\"code-item\">{}<div class=\"card\">{}</div></span>",
                    html, card
                )?;
            }
        }
    }
    // Ends the final line in case the file does not end with a newline character.
    if line_in_progress {
        write!(f, "</code></pre>\n")?;
        writeln!(f, "      </li>")?;
        // line_in_progress = false;  // Assignment would be unused.
    }

    writeln!(f, "    </ol>")?;
    writeln!(f, "  </body>")?;
    writeln!(f, "</html>")?;
    Ok(())
}

// TODO tokenized debug file (step before annotation)

/// TODO
fn annotation_card(token: &AnnotatedToken) -> Option<String> {
    match token.token() {
        Lexeme::Text(token_info) => {
            let html = transform_text_to_html(token_info.characters());
            let highlight = if let Some(annotation) = token.annotation() {
                if let Some(highlight) = annotation.highlight() {
                    format!(" {highlight}")
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            let comment_id = if let Some(annotation) = token.annotation() {
                if let Some(comment_id) = annotation.comment_id() {
                    format!(" comment-{comment_id}")
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let start = token_info.start_column();
            let end = token_info.end_column();
            let range_display = if start == end {
                format!("{start}")
            } else {
                format!("{start}&ndash;{end}")
            };

            let card = format!("<div>{range_display}</div>",);
            Some(format!(
                "<span class=\"code-item{highlight}{comment_id}\">{html}<div class=\"card\">{card}</div></span>",
            ))
        }
        _ => None,
    }
}

/// TODO
pub fn write_annotated_debug_file(
    annotated_tokens: &AnnotatedFile,
    output: &Path,
) -> std::io::Result<()> {
    let mut f = File::create(output)?;
    writeln!(f, "<!DOCTYPE html>")?;
    writeln!(f, "<html lang=\"en\">")?;
    writeln!(f, "{HTML_HEAD}")?;
    writeln!(f, "  <body>")?;
    writeln!(f, "    <ol>")?;
    let mut line_in_progress = false;
    for annotated_token in annotated_tokens.tokens() {
        if !line_in_progress {
            writeln!(f, "      <li>")?;
            write!(f, "        <pre><code>")?;
            line_in_progress = true;
        }
        match annotated_token.token() {
            Lexeme::LineBreak(_token_info) => {
                write!(f, "</code></pre>\n")?;
                writeln!(f, "      </li>")?;
                line_in_progress = false;
            }
            Lexeme::Whitespace(token_info) => {
                write!(f, "{}", transform_text_to_html(token_info.characters()))?;
            }
            Lexeme::Text(_token_info) => {
                write!(f, "{}", annotation_card(annotated_token).unwrap())?;
            }
        }
    }
    // Ends the final line in case the file does not end with a newline character.
    if line_in_progress {
        write!(f, "</code></pre>\n")?;
        writeln!(f, "      </li>")?;
        // line_in_progress = false;  // Assignment would be unused.
    }

    writeln!(f, "    </ol>")?;
    writeln!(f, "  </body>")?;
    writeln!(f, "</html>")?;
    Ok(())
}
