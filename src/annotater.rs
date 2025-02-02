/* Annotates a tokenized file produced by the lexer. */

use crate::lexer::{Lexeme, LexemeFile};

/// TODO
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Annotation {
    /// The class name used for syntax highlighting this token.
    highlight: Option<String>,
    /// The Id number for a comment's opening or closing token.
    comment_id: Option<usize>,
}

impl Annotation {
    /// Returns the name of the class used for syntax highlighting this token.
    pub fn highlight(&self) -> Option<&str> {
        self.highlight.as_ref().map(|s| &s[..])
    }

    /// Returns the id of the comment, if present.
    pub fn comment_id(&self) -> Option<usize> {
        self.comment_id
    }
}

/// A token with annotations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnnotatedToken {
    /// The base token.
    token: Lexeme,
    /// Annotated information about the token, if present.
    annotation: Option<Annotation>,
}

impl AnnotatedToken {
    /// Returns a reference to the underlying token.
    pub fn token(&self) -> &Lexeme {
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
    /// The number of pairs of matching comment delimiters.
    num_matched_comments: usize,
}

impl AnnotatedFile {
    /// Returns the number of matching comment delimiters in this file.
    pub fn num_comments(&self) -> usize {
        self.num_matched_comments
    }

    /// TODO
    pub fn annotate(tokenized_file: &LexemeFile) -> Self {
        AnnotationBuilder::new(tokenized_file).build()
    }

    /// Reference to the annotated tokens of this file.
    pub fn tokens(&self) -> &Vec<AnnotatedToken> {
        &self.tokens
    }
}

/// TODO
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AnnotationBuilder<'a> {
    index: usize,
    comment_id: usize,
    num_matched_comments: usize,
    /// The first `usize` is the index in `annotated_tokens` of the open comment token.
    /// The second `usize` is the comment id of the comment.
    open_comments: Vec<(usize, usize)>,
    original_tokens: &'a LexemeFile,
    annotated_tokens: Vec<AnnotatedToken>,
}

impl<'a> AnnotationBuilder<'a> {
    fn new(original_tokens: &'a LexemeFile) -> Self {
        Self {
            index: 0,
            comment_id: 0,
            num_matched_comments: 0,
            open_comments: vec![],
            original_tokens,
            annotated_tokens: Vec::with_capacity(original_tokens.lexemes().len()),
        }
    }

    fn step(&mut self) -> bool {
        debug_assert!(self.index < self.original_tokens.lexemes().len());
        // TODO
        let token = &self.original_tokens.lexemes()[self.index];

        if let Lexeme::Text(token_info) = token {
            match token_info.characters() {
                "/*" => {
                    let annotated_token = AnnotatedToken {
                        token: token.clone(),
                        annotation: Some(Annotation {
                            highlight: Some(String::from("comment")),
                            comment_id: Some(self.comment_id),
                        }),
                    };
                    self.annotated_tokens.push(annotated_token);
                    self.open_comments.push((self.index, self.comment_id));
                    self.comment_id += 1;
                }
                "*/" => {
                    if let Some((index, id)) = self.open_comments.pop() {
                        // TODO add comment index to open token
                        self.num_matched_comments += 1;
                        self.annotated_tokens.push(AnnotatedToken {
                            token: token.clone(),
                            annotation: Some(Annotation {
                                highlight: Some(String::from("comment")),
                                comment_id: Some(id),
                            }),
                        })
                    } else {
                        // TODO handle mismatched comments properly, for now just avoid highlighting
                        self.annotated_tokens.push(AnnotatedToken {
                            token: token.clone(),
                            annotation: None,
                        })
                    }
                }
                _ => {
                    let annotation = if self.open_comments.is_empty() {
                        None
                    } else {
                        Some(Annotation {
                            highlight: Some(String::from("comment")),
                            comment_id: None,
                        })
                    };
                    self.annotated_tokens.push(AnnotatedToken {
                        token: token.clone(),
                        annotation,
                    })
                }
            }
        } else {
            self.annotated_tokens.push(AnnotatedToken {
                token: token.clone(),
                annotation: None,
            })
        }
        self.index += 1; // Update the index for the next step.
                         // Return whether the index is at the end of the file.
        self.index != self.original_tokens.lexemes().len()
    }

    fn build(mut self) -> AnnotatedFile {
        for _ in 0..self.original_tokens.lexemes().len() {
            self.step();
        }
        // TODO cleanup
        AnnotatedFile {
            tokens: self.annotated_tokens,
            num_matched_comments: self.num_matched_comments,
        }
    }
}
