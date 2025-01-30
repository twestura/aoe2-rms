//! Main entry point for running tools for working with RMS files.
//!
//! This API is subject to change, be sure to check from version to version,
//! as "simply running the code" may produce different effects as the project
//! matures.

/// Runs the application to transform a map script to a html file.
/// Accepts a single input: name of the file in the `maps` folder to transform.
/// The output is written to the `out` folder using the same filename
/// as the input file.
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
    println!("Hello, world!");
    // TODO
}
