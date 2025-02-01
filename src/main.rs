//! Main entry point for running tools for working with RMS files.
//!
//! This API is subject to change, be sure to check from version to version,
//! as "simply running the code" may produce different effects as the project
//! matures.

use std::io::Write;
use std::{fs::OpenOptions, path::PathBuf, process};

use aoe2_rms::{annotater::AnnotatedFile, html_writer, lexer};

/// Runs the application to transform a map script to a html file.
/// Accepts as input the names of the files in the `maps` folder to transform.
/// The output is written to the `out` folder using the same filename
/// as each input file, adding a `.html` file extension.
/// If a file name does not exist in the `maps` folder, an error message
/// stating such is printed to standard error.
///
/// If no input is supplied, all files in the `maps` folder are transformed.
///
/// All maps must be directly in the `maps` folder, nesting in subdirectories
/// is not supported.
///
/// Copies the `style/style.css` file to `out`.
/// If the `style/style.css` folder is missing, an error message is printed to
/// standard error and no files are transformed.
fn main() {
    // Skips the first argument, which is always present.
    let args = std::env::args().skip(1);
    let mut files = vec![];
    if args.len() == 0 {
        for result in std::fs::read_dir("maps/").unwrap() {
            match result {
                Ok(entry) => {
                    if entry.path().is_file() {
                        files.push(entry.path())
                    }
                }
                Err(e) => eprintln!("{e}"),
            }
        }
    } else {
        for arg in args {
            let mut path = PathBuf::with_capacity(2);
            path.push("maps");
            path.push(arg);
            if path.is_file() {
                files.push(path);
            } else {
                eprintln!("`{}` is not an existing file.", path.display());
                path.set_extension("rms");
                if path.is_file() {
                    eprintln!("Did you mean `{}`?", path.display());
                }
            }
        }
    }

    // Copies the style CSS file.
    if let Err(e) = std::fs::copy("style/style.css", "out/style.css") {
        eprintln!("Could not copy `style/style.css` to `out`.\n{e}");
        process::exit(1);
    }

    // Transforms the map files.
    let mut max_comments = 0;
    for path in files {
        let tokens = match lexer::tokenize(&path) {
            Ok(ts) => ts,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };
        let mut pb = PathBuf::from("out");
        pb.push(path.file_name().unwrap());
        pb.set_extension("html");
        let annotated_file = AnnotatedFile::annotate(&tokens);
        max_comments = max_comments.max(annotated_file.num_comments());
        if let Err(e) = html_writer::write_annotated_debug_file(&annotated_file, &pb) {
            println!("{e}");
        }
    }

    // Writes comment match highlight classes to the copied css file.
    let mut css_file = match OpenOptions::new().append(true).open("out/style.css") {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Could not open output css file.\n{e}");
            process::exit(1);
        }
    };
    if max_comments > 0 {
        // Writes a blank line before the comments.
        if let Err(e) = writeln!(css_file, "") {
            eprintln!("Could not write to output css file.\n{e}");
            process::exit(1);
        }
    }
    for i in 0..max_comments {
        if let Err(e) = writeln!(
            css_file,
            ":has(.comment-{i}):hover .comment-{i} {{\n  background-color: #5f5f5f;\n}}\n"
        ) {
            eprintln!("Could not write to output css file.\n{e}");
            process::exit(1);
        }
    }
}
