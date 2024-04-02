use std::{cmp::max, collections::HashSet, hash::Hash};

use itertools::{iproduct, Itertools};

use crate::sudoku::Sudoku;

mod indexed;
use indexed::IndexedMatrixSolver;
mod pointed;
use pointed::PointedMatrixSolver;

use self::indexed::{Unchecked, UncheckedContainerType, VecContainerType};

trait MatrixSolver {
    fn solve<'a, Label, I, J>(rows: &'a I) -> Option<Vec<&'a Label>>
    where
        Label: 'a + Eq + Hash,
        &'a I: 'a + IntoIterator<Item = &'a (Label, J)>,
        &'a J: 'a + IntoIterator<Item = &'a usize>;
}

trait DancingMatrix {
    fn compute_size<'a, Label: 'a, I, J>(rows: &'a I) -> (usize, usize, usize)
    where
        &'a I: 'a + IntoIterator<Item = &'a (Label, J)>,
        &'a J: 'a + IntoIterator<Item = &'a usize>,
    {
        let (nrows, ncols, ncells) =
            rows.into_iter()
                .fold((0, 0, 0), |(nrows, ncols, ncells), (_, cols)| {
                    let (maxcol, nelems) = cols
                        .into_iter()
                        .fold((0, 0), |(m, n), &e| (max(m, e), n + 1));
                    (nrows + 1, max(ncols, 1 + maxcol), ncells + nelems)
                });

        (nrows, ncols, ncells)
    }
}

pub fn solve_indexed_vec(sudoku: &Sudoku) -> Sudoku {
    solve::<IndexedMatrixSolver<VecContainerType>>(sudoku, "indexed_vec")
}

pub fn solve_indexed_unchecked(sudoku: &Sudoku) -> Sudoku {
    solve::<IndexedMatrixSolver<UncheckedContainerType>>(sudoku, "indexed_unchecked")
}

pub fn solve_pointed(sudoku: &Sudoku) -> Sudoku {
    solve::<PointedMatrixSolver>(sudoku, "pointed")
}

fn solve<Solver: MatrixSolver>(sudoku: &Sudoku, name: &str) -> Sudoku {
    let s = sudoku.size() * sudoku.size();

    let fcols = |i, j, k| {
        let (b, _) = sudoku.box_index(i, j);
        vec![
            i * sudoku.size() + k,
            j * sudoku.size() + k + s,
            b * sudoku.size() + k + 2 * s,
            i * sudoku.size() + j + 3 * s,
        ]
    };

    let givens: Vec<(usize, usize, usize)> = iproduct![0..sudoku.size(), 0..sudoku.size()]
        .map(|(i, j)| (i, j, sudoku[(i, j)]))
        .filter(|(_, _, k)| *k != 0)
        .map(|(i, j, k)| (i, j, k as usize - 1))
        .collect();

    let used: HashSet<_> = givens
        .iter()
        .flat_map(|(i, j, k)| fcols(*i, *j, *k))
        .collect();
    let givens: HashSet<_> = givens.iter().collect();

    let rows: Vec<_> = (0..3)
        .map(|_| (0..sudoku.size()))
        .multi_cartesian_product()
        .map(|v| {
            let &[i, j, k] = &*v else { panic!() };
            ((i, j, k), fcols(i, j, k))
        })
        .filter(|(label, cols)| givens.contains(label) || !cols.iter().any(|c| used.contains(c)))
        .collect();

    //let s = sudoku.size();
    //debug_assert!(rows.iter().count() == s * s * s);
    //debug_assert!(rows.iter().filter(|((i, _, _), _)| *i == 0).count() == s * s);

    let solution = Solver::solve(&rows);

    let mut solved = sudoku.clone();
    let name = solved.name().clone() + " (Solved by " + name + ")";
    solved.rename(&name);

    for &(i, j, k) in solution.unwrap_or_else(|| panic!("Solücíon not found!")) {
        assert!(solved[(i, j)] == 0 || solved[(i, j)] == (k + 1) as u8);
        solved[(i, j)] = (k + 1) as u8;
    }

    solved
}

#[cfg(test)]
mod tests {
    use test::Bencher;

    use crate::dancer::*;
    use crate::sudoku;

    #[bench]
    fn bench_indexed_vec_dancer_9x9(b: &mut Bencher) {
        let puzzles = sudoku::test_sudokus();

        b.iter(|| {
            for puzzle in puzzles {
                if puzzle.size() == 9 {
                    super::solve_indexed_vec(puzzle);
                }
            }
        });
    }

    #[bench]
    fn bench_indexed_vec_dancer_all(b: &mut Bencher) {
        let puzzles = sudoku::test_sudokus();

        b.iter(|| {
            for puzzle in puzzles {
                super::solve_indexed_vec(puzzle);
            }
        });
    }

    #[bench]
    fn bench_indexed_unchecked_dancer_9x9(b: &mut Bencher) {
        let puzzles = sudoku::test_sudokus();

        b.iter(|| {
            for puzzle in puzzles {
                if puzzle.size() == 9 {
                    super::solve_indexed_unchecked(puzzle);
                }
            }
        });
    }

    #[bench]
    fn bench_indexed_unchecked_dancer_all(b: &mut Bencher) {
        let puzzles = sudoku::test_sudokus();

        b.iter(|| {
            for puzzle in puzzles {
                super::solve_indexed_unchecked(puzzle);
            }
        });
    }

    #[bench]
    fn bench_pointed_dancer_9x9(b: &mut Bencher) {
        let puzzles = sudoku::test_sudokus();

        b.iter(|| {
            for puzzle in puzzles {
                if puzzle.size() == 9 {
                    super::solve_pointed(puzzle);
                }
            }
        });
    }

    #[bench]
    fn bench_pointed_dancer_all(b: &mut Bencher) {
        let puzzles = sudoku::test_sudokus();

        b.iter(|| {
            for puzzle in puzzles {
                super::solve_pointed(puzzle);
            }
        });
    }

    #[test]
    fn test_simple() {
        let rows = [("a", vec![0, 1]), ("b", vec![0, 1, 2])];
        let solution = IndexedMatrixSolver::<VecContainerType>::solve(&rows);
        println!("solution: {:?}", solution);
    }
}
