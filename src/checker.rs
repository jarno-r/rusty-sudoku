use crate::Sudoku;

#[derive(Copy, Clone)]
pub enum FailureMode {
    Row,
    Col,
    Box,
}

pub fn check_cell(sudoku: &Sudoku, i: usize, j: usize) -> Option<(usize, usize, FailureMode)> {
    for k in 0..sudoku.size() {
        if k != i && sudoku[(i, j)] == sudoku[(k, j)] {
            return Some((i, j, FailureMode::Row));
        }

        if k != j && sudoku[(i, j)] == sudoku[(i, k)] {
            return Some((i, j, FailureMode::Col));
        }

        let (b, c) = sudoku.box_index(i, j);
        if k != c && sudoku[(i, j)] == sudoku.box_cell(b, k) {
            return Some((i, j, FailureMode::Box));
        }
    }
    None
}

pub fn check(sudoku: &Sudoku) {
    let mut incomplete = false;
    let mut failure: Option<(usize, usize, FailureMode)> = None;

    'label: for i in 0..sudoku.size() {
        for j in 0..sudoku.size() {
            if sudoku[(i, j)] == 0 {
                incomplete = true;
            } else {
                failure = check_cell(sudoku, i, j);
                if failure.is_some() {
                    break 'label;
                }
            }
        }
    }

    match failure {
        Some((i, j, kind)) => println!(
            "Bad digit {} at {},{}. Repeats in the same {}.",
            sudoku[(i, j)],
            i + 1,
            j + 1,
            match kind {
                FailureMode::Row => "row",
                FailureMode::Col => "column",
                FailureMode::Box => "box",
            }
        ),
        _ => (),
    }

    if incomplete {
        println!("Sudoku is incomplete!");
    }

    if !incomplete && failure.is_none() {
        println!("Sudoku is correct!");
    }
}
