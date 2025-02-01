//! Tools for writing a parsed RMS file to a debugging HTML file.

use std::{fs::File, io::Write, path::Path};

use crate::{
    annotater::{AnnotatedFile, AnnotatedToken},
    lexer::{Token, TokenizedFile},
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

/// TODO
pub fn write_debug_file(tokens: &TokenizedFile, output: &Path) -> std::io::Result<()> {
    let mut f = File::create(output)?;
    writeln!(f, "<!DOCTYPE html>")?;
    writeln!(f, "<html lang=\"en\">")?;
    writeln!(f, "{HTML_HEAD}")?;
    writeln!(f, "  <body>")?;
    writeln!(f, "    <ol>")?;
    let mut line_in_progress = false;
    for token in tokens.tokens() {
        if !line_in_progress {
            writeln!(f, "      <li>")?;
            write!(f, "        <pre><code>")?;
            line_in_progress = true;
        }
        match token {
            Token::LineBreak(_token_info) => {
                write!(f, "</code></pre>\n")?;
                writeln!(f, "      </li>")?;
                line_in_progress = false;
            }
            Token::Whitespace(token_info) => {
                write!(f, "{}", token_info.characters())?;
            }
            Token::Text(token_info) => {
                let html = transform_text_to_html(token_info.characters());
                let highlight = ""; // TODO classes for syntax highlighting
                let start = token_info.start_column();
                let end = token_info.end_column();
                let range_display = if start == end {
                    format!("{start}")
                } else {
                    format!("{start}&ndash;{end}")
                };
                // TODO more information for the card.
                let card = format!("<div>{range_display}</div>",);
                write!(
                    f,
                    "<span class=\"code-item{}\">{}<div class=\"card\">{}</div></span>",
                    highlight, html, card
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

/// TODO
fn annotation_card(token: &AnnotatedToken) -> Option<String> {
    match token.token() {
        Token::Text(token_info) => {
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
            Token::LineBreak(_token_info) => {
                write!(f, "</code></pre>\n")?;
                writeln!(f, "      </li>")?;
                line_in_progress = false;
            }
            Token::Whitespace(token_info) => {
                write!(f, "{}", transform_text_to_html(token_info.characters()))?;
            }
            Token::Text(_token_info) => {
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
