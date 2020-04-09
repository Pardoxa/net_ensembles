
use crate::sw_graph::SwEdgeIterNeighbors;
use crate::{Node, AdjContainer};
use std::marker::PhantomData;

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


/// Iterator over indices stored in adjecency list
pub struct ContainedIter<'a, T, A>
where T: Node,
      A: AdjContainer<T>
{
    vertex_slice: &'a[A],
    phantom: PhantomData<T>

}

impl<'a, T, A> ContainedIter<'a, T, A>
where T: Node,
      A: AdjContainer<T>
{
    /// Create new iterator over vertex slice
    pub fn new(vertex_slice: &'a[A]) -> Self {
        Self {
            vertex_slice,
            phantom: PhantomData::<T>
        }
    }
}

impl<'a, T, A> Iterator for ContainedIter<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {

        let (container, next_slice) = self
            .vertex_slice
            .split_first()?;
        self.vertex_slice = next_slice;
        Some(container.contained())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a, T, A> DoubleEndedIterator for ContainedIter<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{

    fn next_back(&mut self) -> Option<Self::Item> {
        let (container, next_slice) = self
            .vertex_slice
            .split_last()?;
        self.vertex_slice = next_slice;
        Some(container.contained())
    }
}

impl<'a, T, A> ExactSizeIterator for ContainedIter<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{
    fn len(&self) -> usize {
        self.vertex_slice.len()
    }
}
