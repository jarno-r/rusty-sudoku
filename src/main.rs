use std::io::prelude::*;
use std::{
    fs::File,
    io::{self, BufReader, Error},
};

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
struct Args {
    /// Filename of a sudoku puzzle to solve.
    #[arg(short, long)]
    puzzle: String,
}

struct Sudoku {
    width: u8,
    height: u8,
    grid: Vec<u8>,
}

impl Sudoku {
    fn read_from_file(file: &str) -> io::Result<Sudoku> {
        let f = File::open(file)?;
        let f = BufReader::new(f);

        let mut max_width = 0;
        let mut height = 0;
        let mut raw_lines = vec![vec![0u8; 0]];
        for (i, line) in f.lines().enumerate() {
            let line = line?;
            let raw = line.trim_end();

            // Do not count empty lines at the end of file.
            if !raw.is_empty() {
                height = i + 1;
            }

            fn to_num(ascii: u8) -> io::Result<u8> {
                let r = if ascii >= b'0' && ascii <= b'9' {
                    ascii - b'0'
                } else if ascii >= b'a' && ascii <= b'z' {
                    ascii - b'a' + 10
                } else if ascii == b' ' {
                    0
                } else {
                    Err(Error::other(format!(
                        "Invalid character '{}' in sudoku puzzle.",
                        ascii as char
                    )))?
                };
                Ok(r)
            }

            let num = raw
                .to_ascii_lowercase()
                .as_bytes()
                .iter()
                .map(|c| to_num(*c));
        }

        Err(io::Error::other("Couldn't read Sudoku"))
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let sudoku = Sudoku::read_from_file(&args.puzzle)?;

    Ok(())
}
