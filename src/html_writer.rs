//! Tools for writing a parsed RMS file to a debugging HTML file.

use std::path::Path;

use crate::lexer::TokenizedFile;

/// TODO
pub fn write_debug_file(tokens: &TokenizedFile, output: &Path) -> std::io::Result<()> {
    Ok(())
}
