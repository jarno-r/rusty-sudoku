#![feature(test)]
extern crate test;

use std::io;

use clap::Parser;

mod brute;
mod checker;
mod dancer;
mod sudoku;

use sudoku::Sudoku;

/// Simple program to greet a person
#[derive(Parser, Debug)]
struct Args {
    /// Filename of a sudoku puzzle to solve.
    #[arg(short, long)]
    puzzle: String,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    println!("Loading {}", &args.puzzle);
    let sudoku = Sudoku::read_from_file(&args.puzzle)?;
    println!("{}", sudoku);

    let solved = brute::solve(&sudoku);
    println!("{}", solved);
    checker::check(&solved);

    Ok(())
}
