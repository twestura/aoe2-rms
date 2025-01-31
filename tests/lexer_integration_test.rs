//! Integration test for the lexer.

use std::{fs, path::PathBuf};

use aoe2_rms::lexer;

/// Tests that the lexing process preserves enough information to copy a file without changes.
#[test]
fn copy_files() {
    for result in std::fs::read_dir("maps/").unwrap() {
        let path = result.unwrap().path();
        if !path.is_file() {
            continue;
        }
        let source_text = fs::read_to_string(&path).unwrap();
        let tokens = lexer::tokenize(&path).unwrap();
        let mut pb = PathBuf::from("test_output_files");
        pb.push(path.file_name().unwrap());
        tokens.write_to_path(&pb).unwrap();
        let output_text = fs::read_to_string(&pb).unwrap();
        assert_eq!(source_text, output_text);
    }
}
