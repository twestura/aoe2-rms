# aoe2-rms
Map scripting tooling for working with random map scripts for Age of Empires II.

This application is very much an unstable work in progress and is subject to change rapidly and unpredictable.

## Steps

### Initial Lexer

A first step is to work on creating a suite of debugging tools for the lexer/tokenizer.
The initial application works to read in a map file and output a html file for displaying debugging output.
The file that is read is converted to a data structure that is annotated with information about the lexemes.
This information is then output to the html file for viewing on hover over the corresponding components of the file.

- The `maps` folder contains `rms` files to test transforming.
- The `out` folder contains the `html` files that have been transformed.
- The `style` folder contains the `css` file used for styling the html files. This file is copied to the `out` directory when the application is executed.

At first the tokens are annotated simple with their line number and start and end character positions/columns within each line.
Line numbers and columns are 1-indexed (to match the line numbers and column index information listed in the bottom-right of vs code).

### RMS Investigation

We need to investigate some instances of how Aoe2's RMS parser deals with weird syntax cases.
For example, does it have full unicode support?
Which characters are considered whitespace (e.g. vertical tabs)?
Where is whitespace needed, such as between an attribute and a curly brace `create_object NAME{`?
How does Aoe2's parser continue or fail at parsing a token when such a character is encountered?

- Unicode characters that are non-ascii whitespace are not treated as whitespace.
- There must be whitespace surrounding tokens, otherwise they are included together.
- In particular, curly braces, zero width spaces, slashes, asterisks, and other characters may be included as identifier names, such as `hello{}{/*}*/_​_`.
