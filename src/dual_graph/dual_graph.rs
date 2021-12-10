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

    pub fn iter(&self) -> impl Iterator<Item=&usize>
    {
        self.adj.iter()
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

    pub fn total_vertices(&self) -> usize
    {
        self.graph_1.vertices.len() + self.graph_2.vertices.len()
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

impl<'a, T1, T2, A1, A2> DualGraph<'a, T1, A1, T2, A2>
where A1: AdjContainer<T1>,
    A2: AdjContainer<T2>
{
    /// Depth first search iterator starting at the node corresponding to `index`
    /// 
    /// Note that, this iterator will return indices from both graphs, the corresponding
    /// graph is given by the variant of DualIndex
    pub fn dfs_index(&'a self, index: DualIndex) -> impl 'a + Iterator<Item=DualIndex>
    {
        DfsDualIndex::new(self, index)
    }

    /// Depth first search iterator starting at the node in graph_1 corresponding to `index`
    /// 
    /// Note that, this iterator will return indices from both graphs, the corresponding
    /// graph is given by the variant of DualIndex
    pub fn dfs_1_index(&'a self, index: usize) -> impl 'a + Iterator<Item=DualIndex>
    {
        self.dfs_index(DualIndex::Graph1(index))
    }

    /// Depth first search iterator starting at the node in graph_2 corresponding to `index`
    /// 
    /// Note that, this iterator will return indices from both graphs, the corresponding
    /// graph is given by the variant of DualIndex
    pub fn dfs_2_index(&'a self, index: usize) -> impl 'a + Iterator<Item=DualIndex>
    {
        self.dfs_index(DualIndex::Graph2(index))
    }

    pub fn is_connected(&self) -> bool
    {
        let v1 = self.graph_1.vertex_count() == 0;
        let v2 = self.graph_2.vertex_count() == 0;
        if v1 && v2 {
            true
        } else if  v1 && !v2 {
            match self.graph_2.is_connected()
            {
                Some(r) => r,
                None => unsafe {std::hint::unreachable_unchecked()}
            }
        } else if !v1 && v2 {
            match self.graph_1.is_connected()
            {
                Some(r) => r,
                None => unsafe {std::hint::unreachable_unchecked()}
            }
        } else {
            self.dfs_1_index(0).count() == self.total_vertices()
        }
    }
}

impl<'a, T, A1, A2> DualGraph<'a, T, A1, T, A2>
where A1: AdjContainer<T>,
    A2: AdjContainer<T>
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

    pub fn dfs_contained(&self, index: DualIndex) -> DfsDualContained<T, A1, A2>
    {
        DfsDualContained::new(self, index)
    }

    pub fn dfs_1_contained(&self, index_graph_1: usize) -> DfsDualContained<T, A1, A2>
    {
        DfsDualContained::new(self, DualIndex::Graph1(index_graph_1))
    }

    pub fn dfs_2_contained(&self, index_graph_2: usize) -> DfsDualContained<T, A1, A2>
    {
        DfsDualContained::new(self, DualIndex::Graph2(index_graph_2))
    }
}

/// Index which also stores for which graph the index is
pub enum DualIndex
{
    /// Index for graph_1
    Graph1(usize),
    /// Index for graph_2
    Graph2(usize)
}

pub struct DfsDualIndex<'a, T1, T2, A1, A2>
{
    vertices_graph_1: &'a [A1],
    vertices_graph_2: &'a [A2],
    adj_1: &'a [Adj],
    adj_2: &'a [Adj],
    handled_1: Vec<bool>,
    handled_2: Vec<bool>,
    stack: Vec<DualIndex>,
    marker_t1: PhantomData::<T1>,
    marker_t2: PhantomData::<T2>
}

impl<'a, T1, T2, A1, A2> DfsDualIndex<'a, T1, T2, A1, A2> {
    pub(crate) fn new(
        dual_graph: &'a DualGraph<'a, T1, A1, T2, A2>,
        index: DualIndex
    ) -> Self
    {
        let vertices_graph_1 = &dual_graph.graph_1.vertices;
        let vertices_graph_2 = &dual_graph.graph_2.vertices;
        let mut handled_1 = vec![false; vertices_graph_1.len()];
        let mut handled_2 = vec![false; vertices_graph_2.len()];
        let mut stack: Vec<DualIndex> = Vec::with_capacity(handled_1.len() + handled_2.len());

        match index {
            DualIndex::Graph1(idx) if idx < handled_1.len() => {
                stack.push(index);
                handled_1[idx] = true;
            },
            DualIndex::Graph2(idx) if idx < handled_2.len() => {
                stack.push(index);
                handled_2[idx] = true;
            },
            _ => ()
        };

        Self{
            vertices_graph_1,
            vertices_graph_2,
            handled_1,
            handled_2,
            adj_1: dual_graph.adj_1.as_slice(),
            adj_2: dual_graph.adj_2.as_slice(),
            stack,
            marker_t1: PhantomData::<T1>,
            marker_t2: PhantomData::<T2>
        }
    }
}

impl<'a, T1, T2, A1, A2> Iterator for DfsDualIndex<'a, T1, T2, A1, A2>
where A1: AdjContainer<T1>,
    A2: AdjContainer<T2>
{
    type Item = DualIndex;

    fn next(&mut self) -> Option<Self::Item>
    {
        let index = self.stack.pop()?;
        match index
        {
            DualIndex::Graph1(idx) => {
                let vertex = &self.vertices_graph_1[idx];
                for &i in vertex.neighbors()
                {
                    if !self.handled_1[i] {
                        self.handled_1[i] = true;
                        self.stack.push(DualIndex::Graph1(i));
                    }
                }
                for &i in self.adj_1[idx].iter() {
                    if !self.handled_2[i] {
                        self.handled_2[i] = true;
                        self.stack.push(DualIndex::Graph2(i));
                    }
                }
            },
            DualIndex::Graph2(idx) => {
                let vertex = &self.vertices_graph_2[idx];
                for &i in vertex.neighbors()
                {
                    if !self.handled_2[i] {
                        self.handled_2[i] = true;
                        self.stack.push(DualIndex::Graph2(i));
                    }
                }
                for &i in self.adj_2[idx].iter() {
                    if !self.handled_1[i] {
                        self.handled_1[i] = true;
                        self.stack.push(DualIndex::Graph1(i));
                    }
                }
            }
        }
        Some(index)
    }
}

pub struct DfsDualContained<'a, T, A1, A2>
{
    vertices_graph_1: &'a [A1],
    vertices_graph_2: &'a [A2],
    adj_1: &'a [Adj],
    adj_2: &'a [Adj],
    handled_1: Vec<bool>,
    handled_2: Vec<bool>,
    stack: Vec<DualIndex>,
    marker: PhantomData::<T>
}

impl<'a, T, A1, A2> DfsDualContained<'a, T, A1, A2> {
    pub(crate) fn new(
        dual_graph: &'a DualGraph<'a, T, A1, T, A2>,
        index: DualIndex
    ) -> Self
    {
        let vertices_graph_1 = &dual_graph.graph_1.vertices;
        let vertices_graph_2 = &dual_graph.graph_2.vertices;
        let mut handled_1 = vec![false; vertices_graph_1.len()];
        let mut handled_2 = vec![false; vertices_graph_2.len()];
        let mut stack: Vec<DualIndex> = Vec::with_capacity(handled_1.len() + handled_2.len());

        match index {
            DualIndex::Graph1(idx) if idx < handled_1.len() => {
                stack.push(index);
                handled_1[idx] = true;
            },
            DualIndex::Graph2(idx) if idx < handled_2.len() => {
                stack.push(index);
                handled_2[idx] = true;
            },
            _ => ()
        };

        Self{
            vertices_graph_1,
            vertices_graph_2,
            handled_1,
            handled_2,
            adj_1: dual_graph.adj_1.as_slice(),
            adj_2: dual_graph.adj_2.as_slice(),
            stack,
            marker: PhantomData::<T>
        }
    }
}

impl<'a, T, A1, A2> Iterator for DfsDualContained<'a, T, A1, A2>
where T: 'a,
    A1: AdjContainer<T>,
    A2: AdjContainer<T>
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item>
    {
        match self.stack.pop()?
        {
            DualIndex::Graph1(idx) => {
                let vertex = &self.vertices_graph_1[idx];
                for &i in vertex.neighbors()
                {
                    if !self.handled_1[i] {
                        self.handled_1[i] = true;
                        self.stack.push(DualIndex::Graph1(i));
                    }
                }
                for &i in self.adj_1[idx].iter() {
                    if !self.handled_2[i] {
                        self.handled_2[i] = true;
                        self.stack.push(DualIndex::Graph2(i));
                    }
                }

                Some(vertex.contained())
            },
            DualIndex::Graph2(idx) => {
                let vertex = &self.vertices_graph_2[idx];
                for &i in vertex.neighbors()
                {
                    if !self.handled_2[i] {
                        self.handled_2[i] = true;
                        self.stack.push(DualIndex::Graph2(i));
                    }
                }
                for &i in self.adj_2[idx].iter() {
                    if !self.handled_1[i] {
                        self.handled_1[i] = true;
                        self.stack.push(DualIndex::Graph1(i));
                    }
                }

                Some(vertex.contained())
            }
        }
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


#[derive(Debug, Copy, Clone)]
pub enum AddEdgeError{
    IndexOutOfBounds,
    EdgeExists
}


#[cfg(test)]
mod testing {
    use super::*;
    use crate::{Graph, EmptyNode, SwGraph, CountingNode};

    #[test]
    fn dual_graph_basic_test()
    {
        let mut graph_1 = Graph::<EmptyNode>::new(5);
        graph_1.add_edge(0, 1).unwrap();
        graph_1.add_edge(1, 2).unwrap();
        graph_1.add_edge(3, 4).unwrap();

        assert_eq!(graph_1.is_connected(), Some(false));

        let mut graph_2 = SwGraph::<CountingNode>::new(5);

        graph_2.init_ring_2();
        assert_eq!(graph_2.is_connected(), Some(true));

        let mut dual = DualGraph::new(&graph_1, &graph_2);

        assert!(!dual.is_connected());

        dual.add_edge(1, 0).unwrap();
        assert!(!dual.is_connected());
        dual.add_edge(3, 4).unwrap();
        assert!(dual.is_connected());
    }

    #[test]
    fn dual_graph_dfs_test()
    {
        let mut graph_1 = Graph::<CountingNode>::new(5);
        graph_1.init_ring(2).unwrap();
        let sum_1: usize = graph_1.dfs(0).map(CountingNode::number).sum();
        assert_eq!(sum_1, 10);
        
        let mut graph_2 = Graph::<CountingNode>::new(11);
        graph_2.init_ring(1).unwrap();

        let sum_2: usize = graph_2.dfs(0).map(CountingNode::number).sum();

        assert_eq!(sum_2, 55);

        let mut dual = DualGraph::new(
            &graph_1,
            &graph_2
        );

        let sum_dual_1: usize = dual.dfs_1_contained(0).map(CountingNode::number).sum();
        assert_eq!(sum_1, sum_dual_1);

        let sum_dual_2: usize = dual.dfs_2_contained(0).map(CountingNode::number).sum();
        assert_eq!(sum_2, sum_dual_2);

        dual.add_edge(0, 0).unwrap();

        let sum_of_sum: usize = dual
            .dfs_contained(DualIndex::Graph1(0))
            .map(CountingNode::number)
            .sum();
        
        assert_eq!(sum_1 + sum_2, sum_of_sum);
    }
}