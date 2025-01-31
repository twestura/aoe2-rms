//! Tools for writing a parsed RMS file to a debugging HTML file.

use std::{fs::File, io::Write, path::Path};

use crate::lexer::{Token, TokenizedFile};

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
