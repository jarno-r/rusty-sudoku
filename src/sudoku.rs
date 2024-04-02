use core::fmt;
use std::cmp::{max, min};
use std::fmt::{Display, Formatter};
use std::io::prelude::*;
use std::ops::{Index, IndexMut};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::{env, fs};
use std::{
    fs::File,
    io::{self, BufReader, Error},
};

#[derive(Clone)]
pub struct Sudoku {
    name: String,
    width: usize,
    height: usize,
    box_width: usize,
    box_height: usize,
    grid: Vec<u8>,
}

impl Sudoku {
    #[allow(dead_code)]
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn rename(&mut self, name: &String) {
        self.name.clone_from(name)
    }
    pub fn size(&self) -> usize {
        self.width
    }

    pub fn read_from_file(file: &str) -> io::Result<Sudoku> {
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

    /// Return box index (b,c) of element (i,j),
    /// where b is the box number and c is cell number in the box.
    pub fn box_index(&self, i: usize, j: usize) -> (usize, usize) {
        let c = {
            let y = i % self.box_height;
            let x = j % self.box_width;
            y * self.box_width + x
        };
        let b = {
            let y = i / self.box_height;
            let x = j / self.box_width;
            y * self.width / self.box_width + x
        };

        (b, c)
    }

    /// Reverse of box_index()
    pub fn grid_index(&self, b: usize, c: usize) -> (usize, usize) {
        let by = b / (self.width / self.box_width);
        let bx = b % (self.width / self.box_width);
        let cy = c / self.box_width;
        let cx = c % self.box_width;

        (by * self.box_height + cy, bx * self.box_width + cx)
    }

    /// self[self.grid_index(b,c)]
    pub fn box_cell(&self, b: usize, c: usize) -> u8 {
        self[self.grid_index(b, c)]
    }
}

impl Index<(usize, usize)> for Sudoku {
    type Output = u8;
    fn index(&self, (i, j): (usize, usize)) -> &Self::Output {
        &self.grid[i * self.width + j]
    }
}

impl IndexMut<(usize, usize)> for Sudoku {
    fn index_mut(&mut self, (i, j): (usize, usize)) -> &mut Self::Output {
        &mut self.grid[i * self.width + j]
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
            b as char
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
                    let base = (i * self.box_height + j) * self.width + self.box_width * k;
                    let seg: String = self.grid[base..base + self.box_width]
                        .iter()
                        .map(|c| to_char(*c))
                        .flat_map(|c| [c, ' '])
                        .collect();
                    write!(f, "{}|", seg)?;
                }
            }
            write_spacer(f)?;
        }

        Ok(())
    }
}

#[allow(dead_code)]
pub fn test_sudokus() -> &'static [Sudoku] {
    static CACHE: OnceLock<Vec<Sudoku>> = OnceLock::new();

    CACHE.get_or_init(|| {
        let mut dir = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap());
        dir.push("sudokus");

        let v: Vec<Sudoku> = fs::read_dir(dir)
            .unwrap()
            .flatten()
            .map(|f| Sudoku::read_from_file(f.path().as_os_str().to_str().unwrap()).unwrap())
            .collect();
        v
    })
}
