use std::cmp::max;
use std::fmt::{Display, Formatter};
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
    width: usize,
    height: usize,
    grid: Vec<u8>,
}

impl Sudoku {
    fn read_from_file(file: &str) -> io::Result<Sudoku> {
        fn to_num(ascii: u8) -> io::Result<u8> {
            let r = if ascii.is_ascii_digit() {
                ascii - b'0'
            } else if ascii.is_ascii_lowercase() {
                ascii - b'a' + 10
            } else if ascii == b' ' || ascii == b'\t' {
                0
            } else {
                Err(Error::other(format!(
                    "Invalid character '{}' in sudoku puzzle.",
                    ascii as char
                )))?
            };
            Ok(r)
        }

        let f = File::open(file)?;
        let f = BufReader::new(f);

        let mut max_width = 0;
        let mut height = 0;
        let mut raw_lines = vec![vec![0u8; 0];0];
        let mut nonempty_cols = vec![false; 0];
        for (i, line) in f.lines().enumerate() {
            let line = line?;
            let raw = line.trim_end();

            // Do not count empty lines at the end of file.
            if !raw.is_empty() {
                height = i + 1;
            }

            let row: Vec<u8> = raw
                .to_ascii_lowercase()
                .as_bytes()
                .iter()
                .map(|c| to_num(*c))
                .collect::<io::Result<_>>()?;

            max_width = max(max_width, row.len());
            nonempty_cols.resize(max_width, false);
            for (i, v) in row.iter().enumerate() {
                if *v > 0 {
                    nonempty_cols[i] = true;
                }
            }
            raw_lines.push(row);
        }

        let width = nonempty_cols.iter().filter(|x| **x).count();
        let mut grid = vec![0u8; width * height];

        for i in 0..height {
            let mut p = 0;
            for j in 0..width {
                while !nonempty_cols[p] {
                    p += 1
                }
                if raw_lines[i].len() > p {
                    grid[i * width + j] = raw_lines[i][p];
                }
                p+=1;
            }
        }

        Ok(Sudoku {
            width,
            height,
            grid,
        })
    }
}

impl Display for Sudoku {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fn to_char(n: u8) -> char {
            let b = if n == 0 {
                b' '
            } else if n < 10 {
                n + b'0'
            } else {
                n - 10 + b'a'
            };
            return b as char;
        }

        write!(f,"{} by {}",self.width,self.height)?;
        for i in 0..self.height {
            write!(f, "\n|")?;
            let line:String = self.grid[i * self.width..(i + 1) * self.width]
                .iter()
                .map(|c| to_char(*c))
                .collect();
            write!(f, "{}", line)?;
            write!(f, "|")?;
        }

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    println!("Loading {}",&args.puzzle);
    let sudoku = Sudoku::read_from_file(&args.puzzle)?;
    println!("{}",sudoku);

    Ok(())
}
