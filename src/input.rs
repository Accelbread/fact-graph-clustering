//! Types and parsing functions for input files.
//!
//! Contains the `Document` type as the result of parsing input files.
//!
//! Each supported input format provides a type impementing `InputFormat` which can be used to be
//! generic over the input format.

use std::{
    io::{self, BufRead},
    ops::{Deref, DerefMut},
};

newtype_deref! {
    /// Type representing a document.
    /// A document is a list of paragraphs.
    #[derive(Clone, Debug)]
    pub struct Document(pub Vec<Paragraph>);

    /// Type representing a paragraph.
    /// A paragraph is a list of sentences.
    #[derive(Clone, Debug)]
    pub struct Paragraph(pub Vec<Sentence>);

    /// Type representing a Sentence.
    /// A sentence is a list of terms.
    #[derive(Clone, Debug)]
    pub struct Sentence(pub Vec<Term>);

    /// Type representing a term.
    #[derive(Clone, Debug)]
    pub struct Term(pub String);
}

/// Trait that provides functions for handling input files of a given format.
///
/// Implement this trait to add a new input file format.
pub trait InputFormat {
    /// Parses a file in the input format into a `Document`.
    /// Returns a `std::io:Result`, which contains the `Document` if successful.
    ///
    /// # Examples
    ///
    /// ```
    /// use fact_graph::input::{Document, InputFormat};
    /// use std::io::BufRead;
    ///
    /// fn read_file<I: InputFormat, F: BufRead>(file: F) -> Document {
    ///     match I::parse(file) {
    ///         Ok(d) => d,
    ///         Err(_) => panic!("Error parsing file"),
    ///     }
    /// }
    /// ```
    fn parse<F: BufRead>(file: F) -> io::Result<Document>;
}

/// `InputFormat` implementation for documents in the default newline delimited input format.
///
/// Each non-blank line should contain a sequence of terms corresponding to a sentence in the
/// original source document.
/// Paragraphs are delimited by blank lines.
///
/// # Examples
///
/// ```
/// use fact_graph::input::{Document, InputFormat, NddFile};
/// use std::io::BufReader;
///
/// const INPUT: &str = "\
/// this is the first sentence of the first paragraph
/// this is the second sentence of the first paragraph
///
/// this is the first sentence of the second paragraph
/// this is the second sentence of the second paragraph";
///
/// match NddFile::parse(BufReader::new(INPUT.as_bytes())) {
///    Ok(d) => d,
///    Err(_) => panic!(),
/// };
/// ```
#[allow(missing_debug_implementations, missing_copy_implementations)]
pub struct NddFile;

impl InputFormat for NddFile {
    fn parse<F: BufRead>(file: F) -> io::Result<Document> {
        //let file = BufReader::new(File::open(file)?);
        let mut res = Document(Vec::new());
        let mut in_section = false;
        for line in file.lines() {
            let line = line?;
            if line.is_empty() {
                in_section = false;
                continue;
            }
            if !in_section {
                in_section = true;
                res.push(Paragraph(Vec::new()));
            }
            res.last_mut().unwrap().push(Sentence(
                line.split_whitespace()
                    .map(|t| Term(t.to_string()))
                    .collect(),
            ));
        }
        Ok(res)
    }
}
