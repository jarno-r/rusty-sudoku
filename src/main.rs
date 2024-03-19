use core::fmt;
use std::cmp::{max, min};
use std::fmt::{Display, Formatter};
use std::io::prelude::*;
use std::iter::repeat;
use std::{
    fs::File,
    io::{self, BufReader, Error},
};

use clap::Parser;

mod checker;

/// Simple program to greet a person
#[derive(Parser, Debug)]
struct Args {
    /// Filename of a sudoku puzzle to solve.
    #[arg(short, long)]
    puzzle: String,
}

struct Sudoku {
    name: String,
    width: usize,
    height: usize,
    box_width: usize,
    box_height: usize,
    grid: Vec<u8>,
}

impl Sudoku {
    fn read_from_file(file: &str) -> io::Result<Sudoku> {
        fn to_num(ascii: u8) -> io::Result<u8> {
            let r = if ascii.is_ascii_digit() {
                ascii - b'0'
            } else if ascii.is_ascii_lowercase() {
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

        let f = File::open(file)?;
        let f = BufReader::new(f);

        let mut max_width = 0;
        let mut height = 0;
        let mut raw_lines = vec![vec![0u8; 0]; 0];
        for (i, line) in f.lines().enumerate() {
            let line = line?;
            let raw = line;

            // Do not count empty lines at the end of file.
            if !raw.trim().is_empty() {
                height = i + 1;
            }

            let row: Vec<u8> = raw
                .to_ascii_lowercase()
                .as_bytes()
                .iter()
                .filter(|c| **c != b'\t')
                .map(|c| to_num(*c))
                .collect::<io::Result<_>>()?;

            max_width = max(max_width, row.len());
            raw_lines.push(row);
        }

        let width = max_width;
        // Extend potentially missing rows.
        if height < width {
            height = width;
        }
        let mut grid = vec![0u8; width * height];

        for i in 0..min(height, raw_lines.len()) {
            for j in 0..width {
                if raw_lines[i].len() > j {
                    grid[i * width + j] = raw_lines[i][j];
                }
            }
        }

        if width != height {
            Err(io::Error::other(
                "Width and height of the grid do not match.",
            ))?;
        }

        if width != 9 && width != 16 {
            Err(io::Error::other("Only 9x9 and 16x16 puzzles supported."))?;
        }

        let box_width = if width == 9 { 3 } else { 4 };
        let box_height = box_width;

        Ok(Sudoku {
            name: file.to_string(),
            width,
            height,
            box_width,
            box_height,
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

        write!(f, "{}, {} by {}:", self.name, self.width, self.height)?;

        let write_spacer = |f: &mut Formatter| {
            write!(f, "\n+")?;
            for _ in 0..self.width / self.box_width {
                write!(f, "{}", "--".repeat(self.box_width))?;
                write!(f, "+")?;
            }
            Ok(()) as fmt::Result
        };

        write_spacer(f)?;
        for i in 0..self.height / self.box_height {
            for j in 0..self.box_height {
                write!(f, "\n|")?;
                for k in 0..self.width / self.box_width {
                    let base = (i * self.box_height + j) * self.box_width + self.box_width * k;
                    let seg: String = self.grid[base..base + self.box_width]
                        .iter()
                        .map(|c| to_char(*c))
                        .flat_map(|c| [c,' '])
                        .collect();
                    write!(f, "{}|", seg)?;
                }
            }
            write_spacer(f)?;
        }

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    println!("Loading {}", &args.puzzle);
    let sudoku = Sudoku::read_from_file(&args.puzzle)?;
    println!("{}", sudoku);

    Ok(())
}
