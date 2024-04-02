use std::{
    cell::Cell, collections::HashSet, marker::PhantomData, ops::{Index, IndexMut}, time::Instant
};

use super::*;

type CellIndex=usize;
//type CellIndex=u16;

trait AbstractContainer {
    type Item: Clone;
    fn new(len: usize) -> Self;
    fn push(&mut self, value: Self::Item);
    fn capacity(&self) -> usize;
    fn len(&self) -> usize;
    fn resize(&mut self, new_len: usize, value: Self::Item);
}

pub trait AbstractContainerType {
    type Container<T: Clone>: AbstractContainer<Item = T>
        + Index<usize, Output = T>
        + IndexMut<usize>;
}

mod vec_container;
pub use vec_container::*;
mod unchecked_container;
pub use unchecked_container::*;

pub struct IndexedMatrixSolver<ContainerType: AbstractContainerType> {
    _p: PhantomData<ContainerType>,
}

impl<ContainerType: AbstractContainerType> MatrixSolver for IndexedMatrixSolver<ContainerType> {
    fn solve<'a, Label, I, J>(rows: &'a I) -> Option<Vec<&'a Label>>
    where
        Label: 'a + Eq + Hash,
        &'a I: 'a + IntoIterator<Item = &'a (Label, J)>,
        &'a J: 'a + IntoIterator<Item = &'a usize>,
    {
        let now = Instant::now();
        let mut m = IndexedDancingMatrix::<'a, Label, ContainerType>::new(rows);
        let a = now.elapsed().as_micros();
        let solution = m.solve();
        let b = now.elapsed().as_micros();

        if !cfg!(test) {
            println!("times: {} {}",a,b);
        }
        
        solution
    }
}

#[derive(Clone, Copy)]
struct IndexedCell {
    left: CellIndex,
    right: CellIndex,
    up: CellIndex,
    down: CellIndex,
    column: CellIndex,
}

struct IndexedDancingMatrix<'a, Label: 'a, ContainerType: AbstractContainerType> {
    cells: ContainerType::Container<IndexedCell>,
    labels: ContainerType::Container<&'a Label>,
    root: CellIndex,
    ncells: usize,
    nrows: usize,
    ncols: usize,
}

impl<'a, Label: 'a, ContainerType: AbstractContainerType> DancingMatrix
    for IndexedDancingMatrix<'a, Label, ContainerType>
{
}

impl<'a, Label, ContainerType: AbstractContainerType>
    IndexedDancingMatrix<'a, Label, ContainerType>
{
    fn new<I, J>(rows: &'a I) -> Self
    where
        Label: 'a,
        &'a I: 'a + IntoIterator<Item = &'a (Label, J)>,
        &'a J: 'a + IntoIterator<Item = &'a usize>,
    {
        let (nrows, ncols, ncells) = Self::compute_size(rows);

        const EMPTY_CELL: IndexedCell = IndexedCell {
            left: CellIndex::MAX,
            right: CellIndex::MAX,
            up: CellIndex::MAX,
            down: CellIndex::MAX,
            column: CellIndex::MAX,
        };

        let mut cells = ContainerType::Container::<IndexedCell>::new(ncells + ncols + 1);
        let mut labels = ContainerType::Container::<&'a Label>::new(nrows);
        let mut headers = ContainerType::Container::<CellIndex>::new(ncols + 1);

        // Preallocate first elements of each row so that they have indices 0..nrows
        cells.resize(nrows, EMPTY_CELL);

        // Allocate column headers. First one is the root.
        // Store indices in `headers` for later lookup.
        let root = cells.len() as CellIndex;
        headers.push(root);
        cells.push(IndexedCell {
            left: root,
            right: root,
            up: root,
            down: root,
            column: 0,
        });
        for _ in 1..ncols + 1 {
            let cell = cells.len() as CellIndex;
            headers.push(cell);
            cells.push(IndexedCell {
                left: cells[root.into()].left,
                right: root,
                up: cell,
                down: cell,
                column: 0,
            });
            let crl = cells[root.into()].left;
            cells[crl.into()].right = cell;
            cells[root.into()].left = cell;
        }

        // Insert each row.
        for (i, (label, cols)) in rows.into_iter().enumerate() {
            let i:CellIndex = i.try_into().unwrap();
            labels.push(label);

            // Set for checking that columns are not repeated.
            let mut set = HashSet::new();

            for (n, col) in cols.into_iter().enumerate() {
                assert!(set.insert(col));
                let col:CellIndex=(*col).try_into().unwrap();

                // Get cell index. First one is always equal to the row number.
                let cell = if n == 0 {
                    i
                } else {
                    cells.push(EMPTY_CELL);
                    (cells.len() - 1).try_into().unwrap()
                };

                // Horizontal links
                if cell == i {
                    cells[cell.into()].left = cell;
                    cells[cell.into()].right = cell;
                } else {
                    cells[cell.into()].left = cells[i.into()].left;
                    cells[cell.into()].right = i;
                    let cil = cells[i.into()].left;
                    cells[cil.into()].right = cell;
                    cells[i.into()].left = cell;
                }

                // Vertical links and column headers
                let c = headers[(col + 1).into()];
                cells[cell.into()].column = c;
                cells[cell.into()].up = cells[c.into()].up;
                cells[cell.into()].down = c;
                let ccu = cells[c.into()].up;
                cells[ccu.into()].down = cell;
                cells[c.into()].up = cell;

                // .column is repurposed in the header for counting rows in the column.
                cells[c.into()].column += 1;
            }
        }

        debug_assert!(headers.len() == headers.capacity());
        debug_assert!(ncells + ncols + 1 == cells.capacity());
        debug_assert!(cells.capacity() == cells.len());

        debug_assert!(labels.len() == labels.capacity());

        IndexedDancingMatrix {
            cells,
            labels,
            root,
            ncells,
            nrows,
            ncols,
        }
    }

    fn assert_header(&self, c: usize) {
        debug_assert!(c >= self.nrows && c < self.nrows + self.ncols + 1);
    }

    fn solve(&mut self) -> Option<Vec<&'a Label>> {
        let mut j = self.cells[self.root.into()].right;

        // Matrix is empty, tenemos una solucíon!
        if j == self.root {
            return Some(vec![]);
        }

        // Seek smallest column
        let mut selected = CellIndex::MAX;
        let mut mincount = CellIndex::MAX;
        while j != self.root {
            self.assert_header(j.into());
            if self.cells[j.into()].column < mincount {
                mincount = self.cells[j.into()].column;
                selected = j;

                if mincount == 0 {
                    // Smallest column has no rows, there is no solution.
                    return None;
                }
            }
            j = self.cells[j.into()].right;
        }
        self.assert_header(selected.into());

        let c = selected;

        self.cover(c);

        // Try each row on the selected column.
        let mut r = self.cells[c.into()].down;
        while r != c {
            // Cover all columns on the row.
            let mut j = self.cells[r.into()].right;
            while j != r {
                let cjc = self.cells[j.into()].column;
                self.assert_header(cjc.into());
                self.cover(cjc);
                j = self.cells[j.into()].right;
            }

            // Recurse.
            match self.solve() {
                Some(mut tail) => {
                    // If we have a solution, find which row we are on
                    // by looking up the first node on the row,
                    // which will have index less than `nrows´.
                    // That index corresponds to the row number.
                    while usize::from(r) > self.nrows {
                        r = self.cells[r.into()].left;
                    }
                    let label = self.labels[r.into()];
                    tail.push(label);
                    return Some(tail);
                }
                None => (),
            }

            // Revert changes.
            j = self.cells[r.into()].left;
            while j != r {
                let cjc = self.cells[j.into()].column;
                self.assert_header(cjc.into());
                self.uncover(cjc);
                j = self.cells[j.into()].left;
            }

            r = self.cells[r.into()].down;
        }

        self.uncover(c);

        // No solution found.
        None
    }

    fn cover(&mut self, c: CellIndex) {
        self.assert_header(c.into());
        let ccr = self.cells[c.into()].right;
        self.cells[ccr.into()].left = self.cells[c.into()].left;
        let ccl = self.cells[c.into()].left;
        self.cells[ccl.into()].right = self.cells[c.into()].right;

        let mut i = self.cells[c.into()].down;
        while i != c {
            let mut j = self.cells[i.into()].right;
            while j != i {
                let cjd = self.cells[j.into()].down;
                self.cells[cjd.into()].up = self.cells[j.into()].up;

                let cju = self.cells[j.into()].up;
                self.cells[cju.into()].down = self.cells[j.into()].down;

                let cjc = self.cells[j.into()].column;
                self.assert_header(cjc.into());
                debug_assert!(self.cells[cjc.into()].column > 0);
                self.cells[cjc.into()].column -= 1;

                j = self.cells[j.into()].right;
            }
            i = self.cells[i.into()].down;
        }
    }

    fn uncover(&mut self, c: CellIndex) {
        self.assert_header(c.into());
        let mut i = self.cells[c.into()].up;
        while i != c {
            let mut j = self.cells[i.into()].left;
            while j != i {
                let cjc = self.cells[j.into()].column;
                self.assert_header(cjc.into());
                self.cells[cjc.into()].column += 1;

                let cjd = self.cells[j.into()].down;
                self.cells[cjd.into()].up = j;

                let cju = self.cells[j.into()].up;
                self.cells[cju.into()].down = j;

                j = self.cells[j.into()].left;
            }
            i = self.cells[i.into()].up;
        }

        let crc = self.cells[c.into()].right;
        self.cells[crc.into()].left = c;
        let clc = self.cells[c.into()].left;
        self.cells[clc.into()].right = c;
    }
}
