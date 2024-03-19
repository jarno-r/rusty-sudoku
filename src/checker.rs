use crate::Sudoku;

pub fn check(sudoku: &Sudoku) -> () {
    let mut incomplete = false;
    let mut failure: Option<(usize, usize, &str)> = None;

    for i in 0..sudoku.width {
        for j in 0..sudoku.height {
            if sudoku[(i, j)] == 0 {
                incomplete = true;
            } else {
                for k in 0..sudoku.width {
                    if k != i && sudoku[(i, j)] == sudoku[(k, j)] {
                        failure = Some((i, j, "row"));
                    }

                    if k != j && sudoku[(i, j)] == sudoku[(i, k)] {
                        failure = Some((i, j, "column"));
                    }

                    let (b, c) = sudoku.box_index(i, j);
                    if k != c && sudoku[(i, j)] == sudoku.box_cell(b, k) {
                        failure = Some((i, j, "box"));
                    }
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
            kind
        ),
        _ => (),
    }

    if incomplete {
        println!("Sudoku is incomplete!");
    }

    if !incomplete && failure==None {
        println!("Sudoku is correct!");
    }
}
