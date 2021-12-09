use crate::{GenericGraph, AdjContainer};
use std::marker::PhantomData;

pub struct DualGraph<'a, T1, A1, T2, A2>
{
    graph_1: &'a GenericGraph<T1, A1>,
    graph_2: &'a GenericGraph<T2, A2>,
    adj_1: Vec<Adj>,
    adj_2: Vec<Adj>
}

struct Adj{
    adj: Vec<usize>
}

impl Adj{
    pub fn new() -> Self
    {
        Self{
            adj: Vec::new()
        }
    }

    pub fn is_adjacent(&self, other_index: &usize) -> bool
    {
        self.adj.contains(other_index)
    }

    pub fn add_edge(&mut self, other_index: usize) -> bool
    {
        if self.is_adjacent(&other_index){
            return false;
        }
        self.adj.push(other_index);
        true
    }
}

impl<'a, T1, A1, T2, A2> DualGraph<'a, T1, A1, T2, A2>{
    pub fn new(
        graph_1: &'a GenericGraph<T1, A1>, 
        graph_2: &'a GenericGraph<T2, A2>
    ) -> Self
    {
        let adj_1 = (0..graph_1.vertices.len())
            .map(|_| Adj::new())
            .collect();

        let adj_2 = (0..graph_2.vertices.len())
            .map(|_| Adj::new())
            .collect();

        Self{
            graph_1,
            graph_2,
            adj_1,
            adj_2
        }
    }

    pub fn graph_1(&self) -> &'a GenericGraph<T1, A1>
    {
        self.graph_1
    }

    pub fn graph_2(&self) -> &'a GenericGraph<T2, A2>
    {
        self.graph_2
    }

    pub fn size(&self) -> (usize, usize)
    {
        (
            self.graph_1.vertices.len(),
            self.graph_2.vertices.len()
        )
    }

    pub fn add_edge(
        &mut self, 
        index_graph_1: usize, 
        index_graph_2: usize
    ) -> Result<(), AddEdgeError>
    {
        if self.adj_1.len() <= index_graph_1 || self.adj_2.len() <= index_graph_2 {
            return Err(AddEdgeError::IndexOutOfBounds);
        }
        unsafe {
            if !self.adj_1
                .get_unchecked_mut(index_graph_1)
                .add_edge(index_graph_2)
            {
                return Err(AddEdgeError::EdgeExists)   
            }
            
            if !self.adj_2
                .get_unchecked_mut(index_graph_2)
                .add_edge(index_graph_1)
            {
                unreachable!()
            } else {
                Ok(())
            }
        }

    }
}

impl<'a, T1, A1, T2, A2> DualGraph<'a, T1, A1, T2, A2>
where A1: AdjContainer<T1>
{
    pub fn degree_1(&self, index: usize) -> Option<usize>
    {
        self.graph_1.degree(index)
            .map(|d| d + self.adj_1[index].adj.len())
    }
}

impl<'a, T1, A1, T2, A2> DualGraph<'a, T1, A1, T2, A2>
where A2: AdjContainer<T2>
{
    pub fn degree_2(&self, index: usize) -> Option<usize>
    {
        self.graph_2.degree(index)
            .map(|d| d + self.adj_2[index].adj.len())
    }
}

impl<'a, T, A> DualGraph<'a, T, A, T, A>
where A: AdjContainer<T>
{
    /// Iterate over all neighbors of the node corresponding to
    /// index from graph_1. 
    /// Note, that this also includes possible neighbors in the second 
    /// graph
    pub fn contained_iter_neighbors_1(&'a self, index: usize) -> impl Iterator<Item=&'a T>
    {
        self.graph_1
            .contained_iter_neighbors(index)
            .chain(
                NContainedIter2::new(
                    self.graph_2.vertices.as_slice(), 
                    self.adj_1[index].adj.as_slice()
                )
            )
    }

    /// Iterate over all neighbors of the node corresponding to
    /// index from graph_1. 
    /// Note, that this also includes possible neighbors in the second 
    /// graph
    pub fn contained_iter_neighbors_2(&'a self, index: usize) -> impl Iterator<Item=&'a T>
    {
        self.graph_2
            .contained_iter_neighbors(index)
            .chain(
                NContainedIter2::new(
                    self.graph_1.vertices.as_slice(), 
                    self.adj_2[index].adj.as_slice()
                )
            )
    }
}

struct NContainedIter2<'a, T, A>
where A: AdjContainer<T>
{
    vertex_slice: &'a [A],
    index_iter: &'a [usize],
    phantom: PhantomData<T>
}

impl<'a, T, A> NContainedIter2<'a, T, A>
where A: AdjContainer<T>
{
    pub fn new(
        vertex_slice: &'a [A],
        index_iter: &'a [usize]
    ) -> Self
    {
        Self{
            vertex_slice,
            index_iter,
            phantom: PhantomData::<T>
        }
    }
}

impl<'a, T, A> Iterator for NContainedIter2<'a, T, A>
where A: AdjContainer<T>,
    T: 'a
{
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item>
    {
        let (first, rest) = self.index_iter.split_first()?;
        self.index_iter = rest;
        Some(
            self.vertex_slice[*first]
                .contained()
        )
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>)
    {
        (self.index_iter.len(), Some(self.index_iter.len()))
    }

    #[inline]
    fn count(self) -> usize
    {
        self.index_iter.len()
    }
}


pub enum AddEdgeError{
    IndexOutOfBounds,
    EdgeExists
}
