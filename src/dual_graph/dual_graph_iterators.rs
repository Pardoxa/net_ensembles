use std::marker::PhantomData;
use super::*;
use crate::AdjContainer;

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

pub(crate) struct NContainedIter2<'a, T, A>
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