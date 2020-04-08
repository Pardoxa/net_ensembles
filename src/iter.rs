
use crate::sw_graph::SwEdgeIterNeighbors;

/// # Wrapper for iterators
/// * intended to use for iterating over neighbors of `AdjContainer`
/// * Iterator returns `&u32`
pub enum IterWrapper<'a>{
    /// contains generic slice iter
    GenericIter(std::slice::Iter::<'a,u32>),
    /// contains iter from sw implementation
    SwIter(SwEdgeIterNeighbors::<'a>),
}

impl<'a> Iterator for IterWrapper<'a> {
    type Item = &'a u32;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::GenericIter(iter) => iter.next(),
            Self::SwIter(iter)      => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.len();
        (size, Some(size))
    }
}

/// # Number of neighbors is known
impl<'a> ExactSizeIterator for IterWrapper<'a> {
    fn len(&self) -> usize {
        match self {
            Self::GenericIter(iter) => iter.len(),
            Self::SwIter(iter)      => iter.len(),
        }
    }
}

impl<'a> DoubleEndedIterator for IterWrapper<'a> {

    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::GenericIter(iter) => iter.next_back(),
            Self::SwIter(iter)      => iter.next_back(),
        }

    }
}


impl<'a> IterWrapper<'a> {
    /// Create new `IterWrapper` from generic slice iterator
    pub fn new_generic(iter: std::slice::Iter::<'a,u32>) -> Self {
        Self::GenericIter(iter)
    }

    /// Create new `IterWrapper` from `SwEdgeIterNeighbors`
    pub fn new_sw(iter: SwEdgeIterNeighbors<'a>) -> Self {
        Self::SwIter(iter)
    }
}
