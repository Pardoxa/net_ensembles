//! Contains definitions of a few iterators. Not All of them though.
//!
//! They are returned by a few of the methods, you do not need to build them yourself.

use crate::{sw_graph::SwEdgeIterNeighbors, traits::*};
use std::marker::PhantomData;
use core::iter::FusedIterator;

/// # Wrapper for iterators
/// * intended to use for iterating over neighbors of `AdjContainer`
/// * Iterator returns `&u32`
pub enum IterWrapper<'a>{
    /// contains generic slice iter
    GenericIter(std::slice::Iter::<'a, usize>),
    /// contains iter from sw implementation
    SwIter(SwEdgeIterNeighbors::<'a>),
}

impl<'a> FusedIterator for IterWrapper<'a> { }

impl<'a> Iterator for IterWrapper<'a> {
    type Item = &'a usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::GenericIter(iter) => iter.next(),
            Self::SwIter(iter)      => iter.next(),
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        match self {
            Self::GenericIter(iter) => iter.nth(n),
            Self::SwIter(iter)      => iter.nth(n),
        }
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        match self {
            Self::GenericIter(iter) => iter.fold(init, f),
            Self::SwIter(iter)      => iter.fold(init, f),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.len();
        (size, Some(size))
    }
}

/// # Number of neighbors is known
impl<'a> ExactSizeIterator for IterWrapper<'a> {
    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::GenericIter(iter) => iter.len(),
            Self::SwIter(iter)      => iter.len(),
        }
    }
}

impl<'a> DoubleEndedIterator for IterWrapper<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::GenericIter(iter) => iter.next_back(),
            Self::SwIter(iter)      => iter.next_back(),
        }

    }
}


impl<'a> IterWrapper<'a> {
    /// Create new `IterWrapper` from generic slice iterator
    #[inline]
    pub fn new_generic(iter: std::slice::Iter::<'a, usize>) -> Self {
        Self::GenericIter(iter)
    }

    /// Create new `IterWrapper` from `SwEdgeIterNeighbors`
    #[inline]
    pub fn new_sw(iter: SwEdgeIterNeighbors<'a>) -> Self {
        Self::SwIter(iter)
    }
}


/// Iterator over additional data stored at each vertex in order of indices
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
    pub(crate) fn new(vertex_slice: &'a[A]) -> Self {
        Self {
            vertex_slice,
            phantom: PhantomData::<T>
        }
    }
}

impl<'a, T, A> FusedIterator for ContainedIter<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{     }

impl<'a, T, A> Iterator for ContainedIter<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {

        let (container, next_slice) = self
            .vertex_slice
            .split_first()?;
        self.vertex_slice = next_slice;
        Some(container.contained())
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.vertex_slice.len() {
            self.vertex_slice = &[];
            None
        } else{
            let (elements, next_slice) = self
                .vertex_slice
                .split_at(n + 1);
            self.vertex_slice = next_slice;

            Some(elements[n].contained())
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a, T, A> DoubleEndedIterator for ContainedIter<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{
    #[inline]
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
    #[inline]
    fn len(&self) -> usize {
        self.vertex_slice.len()
    }
}




///////////////////////////
/// Iterator over each vertex directly connected with start vertex in adjecency list of vertex index
pub struct NContainerIter<'a, T, A>
where T: Node,
      A: AdjContainer<T>
{
    vertex_slice:   &'a[A],
    index_iter:     IterWrapper<'a>,
    phantom:        PhantomData<T>

}

impl<'a, T, A> NContainerIter<'a, T, A>
where T: Node,
      A: AdjContainer<T>
{
    /// Create new iterator
    pub(crate) fn new(vertex_slice: &'a[A], index_iter: IterWrapper::<'a>) -> Self {
        Self {
            vertex_slice,
            index_iter,
            phantom: PhantomData::<T>
        }
    }
}

impl<'a, T, A> FusedIterator for NContainerIter<'a, T, A>
where T: Node + 'a,
      A: AdjContainer<T>
{     }

impl<'a, T, A> Iterator for NContainerIter<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{
    type Item = &'a A;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index_iter.next()?;
        Some(&self.vertex_slice[*index])
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let index = self.index_iter.nth(n)?;
        Some(&self.vertex_slice[*index])
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a, T, A> DoubleEndedIterator for NContainerIter<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let index = self.index_iter.next_back()?;
        Some(&self.vertex_slice[*index])
    }
}

impl<'a, T, A> ExactSizeIterator for NContainerIter<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{
    #[inline]
    fn len(&self) -> usize {
        self.index_iter.len()
    }
}



///////////////////////////
/// Iterator over additional information stored at vertices
/// that are directly connected to specific vertex
pub struct NContainedIter<'a, T, A>
where T: Node,
      A: AdjContainer<T>
{
    vertex_slice:   &'a[A],
    index_iter:     IterWrapper<'a>,
    phantom:        PhantomData<T>

}

impl<'a, T, A> FusedIterator for NContainedIter<'a, T, A>
where T: Node + 'a,
      A: AdjContainer<T>
{     }

impl<'a, T, A> NContainedIter<'a, T, A>
where T: Node,
      A: AdjContainer<T>
{
    /// Create new iterator
    pub(crate) fn new(vertex_slice: &'a[A], index_iter: IterWrapper::<'a>) -> Self {
        Self {
            vertex_slice,
            index_iter,
            phantom: PhantomData::<T>
        }
    }
}

impl<'a, T, A> Iterator for NContainedIter<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index_iter.next()?;
        Some(&self.vertex_slice[*index].contained())
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let index = self.index_iter.nth(n)?;
        Some(&self.vertex_slice[*index].contained())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a, T, A> DoubleEndedIterator for NContainedIter<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let index = self.index_iter.next_back()?;
        Some(&self.vertex_slice[*index].contained())
    }
}

impl<'a, T, A> ExactSizeIterator for NContainedIter<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{

    #[inline]
    fn len(&self) -> usize {
        self.index_iter.len()
    }
}


///////////////////////////
/// Iterator over additional information + indices stored at vertices
/// that are directly connected to specific vertex
pub struct NIContainedIter<'a, T, A>
where T: Node,
      A: AdjContainer<T>
{
    vertex_slice:   &'a[A],
    index_iter:     IterWrapper<'a>,
    phantom:        PhantomData<T>

}

impl<'a, T, A> FusedIterator for NIContainedIter<'a, T, A>
where T: Node + 'a,
      A: AdjContainer<T>
{     }

impl<'a, T, A> NIContainedIter<'a, T, A>
where T: Node,
      A: AdjContainer<T>
{
    /// Create new iterator
    pub(crate) fn new(vertex_slice: &'a[A], index_iter: IterWrapper::<'a>) -> Self {
        Self {
            vertex_slice,
            index_iter,
            phantom: PhantomData::<T>
        }
    }
}

impl<'a, T, A> Iterator for NIContainedIter<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{
    type Item = (usize, &'a T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let index = *self.index_iter.next()?;
        Some((index, &self.vertex_slice[index].contained()))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let index = *self.index_iter.nth(n)?;
        Some((index, &self.vertex_slice[index].contained()))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a, T, A> DoubleEndedIterator for NIContainedIter<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let index = *self.index_iter.next_back()?;
        Some((index, &self.vertex_slice[index].contained()))
    }
}

impl<'a, T, A> ExactSizeIterator for NIContainedIter<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{

    #[inline]
    fn len(&self) -> usize {
        self.index_iter.len()
    }
}



///////////////////////////
/// * same as NContainedIter but mutable
/// * Iterator over mutable additional information stored at vertices
/// that are directly connected to specific vertex
pub struct NContainedIterMut<'a, T, A>
where T: Node,
      A: AdjContainer<T>
{
    vertex_slice:   &'a mut [A],
    index_iter:     IterWrapper<'a>,
    phantom:        PhantomData<T>

}

impl<'a, T, A> NContainedIterMut<'a, T, A>
where T: Node,
      A: AdjContainer<T>
{
    /// Create new iterator
    pub(crate) fn new(vertex_slice: &'a mut[A], index_iter: IterWrapper::<'a>) -> Self {
        Self {
            vertex_slice,
            index_iter,
            phantom: PhantomData::<T>
        }
    }
}


impl<'a, T, A> Iterator for NContainedIterMut<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index_iter.next()?;
        let index = *index as isize;

        assert!(index < self.vertex_slice.len() as isize);

        let ptr = self.vertex_slice.as_mut_ptr();
        let r1: &mut A = unsafe { &mut *ptr.offset(index) };

        Some(r1.contained_mut())
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let index = self.index_iter.nth(n)?;
        let index = *index as isize;

        assert!(index < self.vertex_slice.len() as isize);

        let ptr = self.vertex_slice.as_mut_ptr();
        let r1: &mut A = unsafe { &mut *ptr.offset(index) };

        Some(r1.contained_mut())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }

}

impl<'a, T, A> DoubleEndedIterator for NContainedIterMut<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let index = self.index_iter.next_back()?;

        let index = *index as isize;

        assert!(index < self.vertex_slice.len() as isize);

        let ptr = self.vertex_slice.as_mut_ptr();
        let r1: &mut A = unsafe { &mut *ptr.offset(index) };

        Some(r1.contained_mut())
    }
}

impl<'a, T, A> ExactSizeIterator for NContainedIterMut<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{

    #[inline]
    fn len(&self) -> usize {
        self.index_iter.len()
    }
}


///////////////////////////
/// * same as NContainedIter but mutable
/// * Iterator over mutable additional information stored at vertices
/// that are directly connected to specific vertex
pub struct INContainedIterMut<'a, T, A>
where T: Node,
      A: AdjContainer<T>
{
    vertex_slice:   &'a mut [A],
    index_iter:     IterWrapper<'a>,
    phantom:        PhantomData<T>

}

impl<'a, T, A> INContainedIterMut<'a, T, A>
where T: Node,
      A: AdjContainer<T>
{
    /// Create new iterator
    pub(crate) fn new(vertex_slice: &'a mut[A], index_iter: IterWrapper::<'a>) -> Self {
        Self {
            vertex_slice,
            index_iter,
            phantom: PhantomData::<T>
        }
    }
}


impl<'a, T, A> Iterator for INContainedIterMut<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{
    type Item = (usize, &'a mut T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let index = *self.index_iter.next()?;

        assert!(index < self.vertex_slice.len());

        let ptr = self.vertex_slice.as_mut_ptr();
        let r1: &mut A = unsafe { &mut *ptr.add(index) };

        Some((index, r1.contained_mut()))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let index = *self.index_iter.nth(n)?;

        assert!(index < self.vertex_slice.len());

        let ptr = self.vertex_slice.as_mut_ptr();
        let r1: &mut A = unsafe { &mut *ptr.add(index) };

        Some((index, r1.contained_mut()))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }

}

impl<'a, T, A> DoubleEndedIterator for INContainedIterMut<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let index = *self.index_iter.next_back()?;

        assert!(index < self.vertex_slice.len());

        let ptr = self.vertex_slice.as_mut_ptr();
        let r1: &mut A = unsafe { &mut *ptr.add(index) };

        Some((index, r1.contained_mut()))
    }
}

impl<'a, T, A> ExactSizeIterator for INContainedIterMut<'a, T, A>
where T: 'a + Node,
      A: AdjContainer<T>
{

    #[inline]
    fn len(&self) -> usize {
        self.index_iter.len()
    }
}

///////////////////////////
/// * same as ContainedIter but mutable
/// * Iterator over mutable additional information stored at vertices
pub struct ContainedIterMut<'a, T, A>
where T: Node,
      A: AdjContainer<T>
{
    vertex_iter:   core::slice::IterMut::<'a, A>,
    phantom:       PhantomData<T>

}

impl<'a, T, A> ContainedIterMut<'a, T, A>
where
    T: Node,
    A: AdjContainer<T>,
{
    pub(crate) fn new(vertex_iter: core::slice::IterMut::<'a, A>) -> Self {
        Self{
            vertex_iter,
            phantom: PhantomData::<T>,
        }
    }
}

impl<'a, T, A> FusedIterator for ContainedIterMut<'a, T, A>
where
T: 'a + Node,
A: AdjContainer<T>
{       }

impl<'a, T, A> Iterator for ContainedIterMut<'a, T, A>
where
    T: 'a + Node,
    A: AdjContainer<T>,
{
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.vertex_iter.next()?;

        Some(next.contained_mut())
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let nth = self.vertex_iter.nth(n)?;

        Some(nth.contained_mut())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}


impl<'a, T, A> DoubleEndedIterator for ContainedIterMut<'a, T, A>
where
    T: 'a + Node,
    A: AdjContainer<T>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let last = self.vertex_iter.next_back()?;

        Some(last.contained_mut())
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let nth_back = self.vertex_iter.nth_back(n)?;

        Some(nth_back.contained_mut())
    }
}

impl<'a, T, A> ExactSizeIterator for ContainedIterMut<'a, T, A>
where
    T: 'a + Node,
    A: AdjContainer<T>,
{
    #[inline]
    fn len(&self) -> usize {
        self.vertex_iter.len()
    }
}
