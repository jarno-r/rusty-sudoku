use crate::{checker, Sudoku};

pub fn solve(sudoku: &Sudoku) -> Sudoku {
    let mut sudoku = sudoku.clone();
    let name = sudoku.name().clone() + " (Solved by brute)";
    sudoku.rename(&name);

    fn solve(sudoku: &mut Sudoku) -> bool {
        for i in 0..sudoku.size() {
            for j in 0..sudoku.size() {
                if sudoku[(i, j)] == 0 {
                    for k in 0..sudoku.size() {
                        sudoku[(i, j)] = 1 + k as u8;
                        if checker::check_cell(sudoku, i, j).is_none() && solve(sudoku) {
                            return true;
                        }
                    }
                    sudoku[(i, j)] = 0;
                    return false;
                }
            }
        }
        true
    }

    solve(&mut sudoku);

    sudoku
}

#[cfg(test)]
mod tests {
    use test::Bencher;

    use crate::sudoku;

    #[bench]
    fn bench_brute_9x9(b: &mut Bencher) {
        let puzzles = sudoku::test_sudokus();

        b.iter(|| {
            for puzzle in puzzles {
                // 16x16 sudokus take way too long for the brute-force algorithm.
                if puzzle.size() == 9 {
                    super::solve(puzzle);
                }
            }
        });
    }
}
