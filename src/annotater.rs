/* Annotates a tokenized file produced by the lexer. */

use crate::lexer::{Token, TokenizedFile};

/// TODO
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Annotation {
    /// The class name used for syntax highlighting this token.
    highlight: Option<String>,
}

impl Annotation {
    /// Returns the name of the class used for syntax highlighting this token.
    pub fn highlight(&self) -> Option<&str> {
        self.highlight.as_ref().map(|s| &s[..])
    }
}

/// A token with annotations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnnotatedToken {
    /// The base token.
    token: Token,
    /// Annotated information about the token, if present.
    annotation: Option<Annotation>,
}

impl AnnotatedToken {
    /// Returns a reference to the underlying token.
    pub fn token(&self) -> &Token {
        &self.token
    }
    /// Returns the annotation as an optional reference.
    pub fn annotation(&self) -> Option<&Annotation> {
        self.annotation.as_ref()
    }
}

/// A file of tokens along with their annotations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnnotatedFile {
    /// The annotated tokens corresponding to the file.
    tokens: Vec<AnnotatedToken>,
}

impl AnnotatedFile {
    /// TODO
    pub fn annotate(tokenized_file: &TokenizedFile) -> Self {
        let mut annotated_tokens = Vec::with_capacity(tokenized_file.tokens().len());
        let mut comment_depth = 0;
        for token in tokenized_file.tokens() {
            let mut end_comment = false;
            if let Token::Text(token_info) = token {
                match token_info.characters() {
                    "/*" => {
                        comment_depth += 1;
                    }
                    "*/" => {
                        // TODO handle mismatched comments
                        if comment_depth > 0 {
                            comment_depth -= 1;
                            if comment_depth == 0 {
                                end_comment = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
            let annotation = if comment_depth > 0 || end_comment {
                Some(Annotation {
                    highlight: Some(String::from("comment")),
                })
            } else {
                None
            };
            annotated_tokens.push(AnnotatedToken {
                token: token.clone(),
                annotation,
            })
        }
        Self {
            tokens: annotated_tokens,
        }
    }

    /// Reference to the annotated tokens of this file.
    pub fn tokens(&self) -> &Vec<AnnotatedToken> {
        &self.tokens
    }
}
