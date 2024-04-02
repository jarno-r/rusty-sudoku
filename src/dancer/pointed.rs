use std::{
    alloc::{alloc, dealloc, Layout},
    marker::PhantomData,
    mem::{align_of, size_of},
    ops::Index,
    ptr::{null, null_mut},
};

use super::*;

pub struct PointedMatrixSolver {}

impl MatrixSolver for PointedMatrixSolver {
    fn solve<'a, Label, I, J>(rows: &'a I) -> Option<Vec<&'a Label>>
    where
        Label: 'a + Eq + Hash,
        &'a I: 'a + IntoIterator<Item = &'a (Label, J)>,
        &'a J: 'a + IntoIterator<Item = &'a usize>,
    {
        unsafe {
            let mut m = PointedDancingMatrix::new(rows);
            m.solve()
        }
    }
}

#[repr(C)]
struct BasicNode<'a, Label> {
    up: *mut Node<'a, Label>,
    down: *mut Node<'a, Label>,
    left: *mut Node<'a, Label>,
    right: *mut Node<'a, Label>,
}

#[repr(C)]
struct LabelNode<'a, Label> {
    _r: RegularNode<'a, Label>,
    label: &'a Label,
}

#[repr(C)]
struct HeaderNode<'a, Label> {
    _b: BasicNode<'a, Label>,
    count: usize,
}

#[repr(C)]
struct RegularNode<'a, Label> {
    _b: BasicNode<'a, Label>,
    column: *mut Node<'a, Label>,
}

#[repr(C)]
#[derive(Copy, Clone)]
union Node<'a, Label> {
    b: BasicNode<'a, Label>,
    h: HeaderNode<'a, Label>,
    r: RegularNode<'a, Label>,
    l: LabelNode<'a, Label>,
}

macro_rules! impl_copy_clone {
    { $t:ident } => {
        impl<'a,Label> Clone for $t<'a,Label> { fn clone(&self) -> Self {*self} }
        impl<'a,Label> Copy for $t<'a,Label> {  }
    }
}

impl_copy_clone!(BasicNode);
impl_copy_clone!(RegularNode);
impl_copy_clone!(HeaderNode);
impl_copy_clone!(LabelNode);

struct PointedDancingMatrix<'a, Label> {
    root: *mut Node<'a, Label>,
    labels: NodeArray<LabelNode<'a, Label>, Node<'a, Label>>,
    headers: NodeArray<HeaderNode<'a, Label>, Node<'a, Label>>,
    nodes: NodeArray<RegularNode<'a, Label>, Node<'a, Label>>,
}

impl<'a, Label> DancingMatrix for PointedDancingMatrix<'a, Label> {}

impl<'a, Label> PointedDancingMatrix<'a, Label> {
    unsafe fn new<I, J>(rows: &'a I) -> Self
    where
        &'a I: 'a + IntoIterator<Item = &'a (Label, J)>,
        &'a J: 'a + IntoIterator<Item = &'a usize>,
    {
        debug_assert!(
            align_of::<RegularNode::<'a, Label>>() == align_of::<BasicNode::<'a, Label>>()
        );
        debug_assert!(align_of::<LabelNode::<'a, Label>>() == align_of::<BasicNode::<'a, Label>>());
        debug_assert!(
            align_of::<HeaderNode::<'a, Label>>() == align_of::<BasicNode::<'a, Label>>()
        );
        debug_assert!(align_of::<Node::<'a, Label>>() == align_of::<BasicNode::<'a, Label>>());

        let (nrows, ncols, ncells) = Self::compute_size(rows);

        let labels = NodeArray::<LabelNode<'a, Label>, Node<'a, Label>>::new(nrows);
        let headers = NodeArray::<HeaderNode<'a, Label>, Node<'a, Label>>::new(ncols + 1);
        let nodes = NodeArray::<RegularNode<'a, Label>, Node<'a, Label>>::new(ncells - nrows);

        let root = headers.get(0);
        (*root).b.left = root;
        (*root).b.right = root;
        (*root).b.up = root;
        (*root).b.down = root;
        (*root).h.count = 0;

        for i in 1..ncols + 1 {
            let c = headers.get(i);
            (*c).b.left = (*root).b.left;
            (*c).b.right = root;
            (*(*root).b.left).b.right = c;
            (*root).b.left = c;

            (*c).b.up = c;
            (*c).b.down = c;

            (*c).h.count = 0;
        }

        let mut p = 0;
        for (i, (label, cols)) in rows.into_iter().enumerate() {
            let r: *mut Node<'a, Label> = labels.get(i);
            (*r).l.label = label;
            (*r).b.left = r;
            (*r).b.right = r;

            // Set for checking that columns are not repeated.
            let mut set = HashSet::new();

            for (j, c) in cols.into_iter().enumerate() {
                assert!(set.insert(c));

                let n = if j == 0 {
                    r
                } else {
                    p += 1;
                    nodes.get(p - 1)
                };

                if j != 0 {
                    (*n).b.left = (*r).b.left;
                    (*n).b.right = r;

                    (*(*r).b.left).b.right = n;
                    (*r).b.left = n;
                }

                let h = headers.get(c + 1);
                debug_assert!(headers.contains(h));
                (*n).r.column = h;

                (*n).b.down = h;
                (*n).b.up = (*h).b.up;

                (*(*h).b.up).b.down = n;
                (*h).b.up = n;

                (*h).h.count += 1;
            }
        }

        Self {
            root,
            labels,
            headers,
            nodes,
        }
    }

    unsafe fn solve(&mut self) -> Option<Vec<&'a Label>> {
        let mut h = (*self.root).b.right;

        if h == self.root {
            // Solution found!
            return Some(vec![]);
        }

        // Find column with least rows.
        let mut selected = null_mut();
        let mut maxcount = usize::MAX;
        while h != self.root {
            if (*h).h.count < maxcount {
                maxcount = (*h).h.count;
                selected = h;

                // An empty column. No solution.
                if maxcount == 0 {
                    return None;
                }
            }

            h = (*h).b.right;
        }

        debug_assert!(self.headers.contains(selected));
        self.cover(selected);

        let mut r = (*selected).b.down;
        while r != selected {
            debug_assert!(self.nodes.contains(r) || self.labels.contains(r));

            debug_assert!(self.headers.contains((*r).r.column));
            debug_assert!((*r).r.column == selected);

            let mut c = (*r).b.right;

            while c != r {
                debug_assert!(self.nodes.contains(c) || self.labels.contains(c));
                debug_assert!(self.headers.contains((*c).r.column));
                self.cover((*c).r.column);
                c = (*c).b.right;
            }

            if let Some(mut tail) = self.solve() {
                // Seek the label node.
                while !self.labels.contains(r) {
                    r = (*r).b.left;
                }
                tail.push((*r).l.label);
                return Some(tail);
            }

            c = (*r).b.left;
            while c != r {
                self.uncover((*c).r.column);
                c = (*c).b.left;
            }

            r = (*r).b.down;
        }

        self.uncover(selected);

        None
    }

    unsafe fn cover(&self, h: *mut Node<'a, Label>) -> () {
        debug_assert!(self.headers.contains(h));

        (*(*h).b.right).b.left = (*h).b.left;
        (*(*h).b.left).b.right = (*h).b.right;

        let mut i = (*h).b.down;
        while i != h {
            debug_assert!(self.labels.contains(i) || self.nodes.contains(i));

            let mut j = (*i).b.right;
            while j != i {
                debug_assert!(self.labels.contains(j) || self.nodes.contains(j));

                (*(*j).b.down).b.up = (*j).b.up;
                (*(*j).b.up).b.down = (*j).b.down;

                debug_assert!(self.headers.contains((*j).r.column));
                (*(*j).r.column).h.count -= 1;

                j = (*j).b.right;
            }

            i = (*i).b.down;
        }
    }

    unsafe fn uncover(&self, h: *mut Node<'a, Label>) -> () {
        debug_assert!(self.headers.contains(h));

        let mut i = (*h).b.up;
        while i != h {
            let mut j = (*i).b.left;
            while j != i {
                (*(*j).b.down).b.up = j;
                (*(*j).b.up).b.down = j;

                debug_assert!(self.headers.contains((*j).r.column));
                (*(*j).r.column).h.count += 1;

                j = (*j).b.left;
            }

            i = (*i).b.up;
        }

        (*(*h).b.right).b.left = h;
        (*(*h).b.left).b.right = h;
    }
}

struct NodeArray<T: Copy, Node> {
    ptr: *mut T,
    len: usize,
    end: *mut T,
    _p: PhantomData<Node>,
}

impl<T: Copy, Node> NodeArray<T, Node> {
    unsafe fn new(len: usize) -> Self {
        let align = align_of::<T>();
        let size = size_of::<T>() * len;
        let ptr = alloc(Layout::from_size_align_unchecked(size, align)) as *mut T;

        debug_assert!(size_of::<T>() % align == 0);

        Self {
            _p: PhantomData,
            ptr,
            len,
            end: ptr.offset(len as isize),
        }
    }

    fn contains(&self, p: *const Node) -> bool {
        let p = p as *const T;
        p >= self.ptr && p < self.end
    }

    unsafe fn get(&self, i: usize) -> *mut Node {
        debug_assert!(i < self.len);
        self.ptr.offset(i as isize) as *mut Node
    }
}

impl<T: Copy, Node> Drop for NodeArray<T, Node> {
    fn drop(&mut self) {
        unsafe {
            let align = align_of::<T>();
            let size = self.end.byte_offset_from(self.ptr) as usize;
            dealloc(
                self.ptr as *mut u8,
                Layout::from_size_align_unchecked(size, align),
            )
        };
    }
}
