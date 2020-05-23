use itertools::Itertools;
use rayon::prelude::*;
use std::{
    collections::HashSet,
    env,
    error::Error,
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
    process,
};

type Document = Vec<Vec<Vec<String>>>;

fn main() {
    match env::set_current_dir("workdir") {
        Ok(()) => (),
        Err(e) => error("Unable to enter workdir", e),
    }

    let files: Vec<_> = match fs::read_dir("raw_input") {
        Err(e) => error("Unable to read input directory", e),
        Ok(dir_iter) => match dir_iter.collect() {
            Ok(files) => files,
            Err(e) => error("Error while reading input directory", e),
        },
    };

    files.par_iter().for_each(|file| {
        let reader = BufReader::new(match File::open(file.path()) {
            Ok(f) => f,
            Err(e) => error("Error opening file", e),
        });
        let outpath: PathBuf = ["input".into(), file.file_name()].iter().collect();
        let outfile = match File::create(outpath) {
            Ok(f) => f,
            Err(e) => error("Unable to create output file", e),
        };
        let doc = match preprocess(reader, &stopwords()) {
            Ok(f) => f,
            Err(e) => error("Error during parsing file", e),
        };
        match write_doc(doc, outfile) {
            Ok(f) => f,
            Err(e) => error("Error writing file", e),
        };
    });
}

type Stopwords = HashSet<String>;

fn stopwords() -> Stopwords {
    let stopwords_file = include_str!("stopwords.txt");
    let mut res = HashSet::new();
    for w in stopwords_file.lines() {
        res.insert(w.to_string());
    }
    res
}

fn preprocess<R: BufRead>(input: R, stopwords: &Stopwords) -> Result<Document, io::Error> {
    let mut doc = vec![vec![vec![]]];
    for l in input.lines() {
        let l = l?;
        let line: Vec<_> = l
            .split(|c: char| c.is_whitespace() || c == '-' || c == '—')
            .filter(|s| !s.is_empty())
            .collect();
        if line.is_empty() {
            // empty line, so new paragraph
            // check if current paragraph empty (only one sentence, which is empty)
            if doc.last().unwrap().len() == 1 && doc.last().unwrap().last().unwrap().is_empty() {
                continue;
            }
            // new paragraph
            doc.push(Vec::new());
            // new sentence
            doc.last_mut().unwrap().push(Vec::new());
        }
        for w in line {
            let (w, end) = process_word(w, stopwords);
            if let Some(w) = w {
                doc.last_mut().unwrap().last_mut().unwrap().push(w);
            }
            if end {
                // check current sentence not empty
                if !doc.last().unwrap().last().unwrap().is_empty() {
                    doc.last_mut().unwrap().push(Vec::new());
                }
            }
        }
    }
    // Might have extra empty vecs at end
    if doc.last().unwrap().last().unwrap().is_empty() {
        doc.last_mut().unwrap().pop();
    }
    if doc.last().unwrap().is_empty() {
        doc.pop();
    }
    Ok(doc)
}

fn process_word(word: &str, stopwords: &Stopwords) -> (Option<String>, bool) {
    let end = match word.chars().last() {
        Some('.') | Some('?') | Some('!') => true,
        _ => false,
    };
    let word = word.to_lowercase();
    let word = word.replace(|c: char| !c.is_alphabetic(), "");
    if word.is_empty() || stopwords.contains(&word) {
        return (None, end);
    }
    (Some(word), end)
}

fn write_doc<W: Write>(doc: Document, mut out: W) -> Result<(), io::Error> {
    let mut first_para = true;
    for p in doc {
        if first_para {
            first_para = false;
        } else {
            write!(out, "\n\n")?;
        }
        let mut first_sent = true;
        for s in p {
            if first_sent {
                first_sent = false;
            } else {
                writeln!(out)?;
            }
            for w in s.iter().intersperse(&" ".into()) {
                write!(out, "{}", w)?;
            }
        }
    }
    Ok(())
}

fn error(message: &str, err: impl Error) -> ! {
    eprintln!("{}: {}", message, err);
    process::exit(1);
}
