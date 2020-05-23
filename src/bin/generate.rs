use fact_graph::{
    config::construct_method,
    input::{InputFormat, NddFile},
};
use rayon::prelude::*;
use std::{
    env,
    error::Error,
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
    process,
};

fn main() {
    type Format = NddFile;

    match env::set_current_dir("workdir") {
        Ok(()) => (),
        Err(e) => error("Unable to enter workdir", e),
    }

    let files: Vec<_> = match fs::read_dir("input") {
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
        let document = match Format::parse(reader) {
            Ok(d) => d,
            Err(e) => error("Error parsing file", e),
        };
        let graph = construct_method(&document);
        let outpath: PathBuf = ["graphs".into(), file.file_name()].iter().collect();
        let outfile = match File::create(outpath) {
            Ok(f) => f,
            Err(e) => error("Unable to create output file", e),
        };
        match serde_json::to_writer(outfile, &graph) {
            Ok(()) => (),
            Err(e) => error("Failed to serialize data.", e),
        }
    });
}

fn error(message: &str, err: impl Error) -> ! {
    eprintln!("{}: {}", message, err);
    process::exit(1);
}
