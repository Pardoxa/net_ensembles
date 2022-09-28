use crate::{graph::NodeContainer, AdjList};

use {
    crate::{GenericGraph, AdjContainer},
    super::dual_graph_iterators::*,
    crate::iter::NContainedIterMut,
    std::ops::{Deref, DerefMut}
};

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

pub type MultiDualGraph<T1, A1, T2, A2> = DualGraph<Adj, T1, A1, T2, A2>;
pub type SingleDualGraph<T1, A1, T2, A2> = DualGraph<AdjSingle, T1, A1, T2, A2>;
pub type DefaultSDG<T1, T2> = SingleDualGraph<T1, NodeContainer<T1>, T2, NodeContainer<T2>>;


#[derive(Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct DualGraph<ADJ, T1, A1, T2, A2>
{
    pub(crate) graph_1: GenericGraph<T1, A1>,
    pub(crate) graph_2: GenericGraph<T2, A2>,
    pub(crate) adj_1: Vec<ADJ>,
    pub(crate) adj_2: Vec<ADJ>
}

impl<ADJ, T1, A1, T2, A2> DualGraph<ADJ, T1, A1, T2, A2>
where ADJ: AdjTrait{
    pub fn new(
        graph_1: GenericGraph<T1, A1>, 
        graph_2: GenericGraph<T2, A2>
    ) -> Self
    {
        let adj_1 = (0..graph_1.vertices.len())
            .map(|_| ADJ::new())
            .collect();

        let adj_2 = (0..graph_2.vertices.len())
            .map(|_| ADJ::new())
            .collect();

        Self{
            graph_1,
            graph_2,
            adj_1,
            adj_2
        }
    }

    pub fn graph_1(&self) -> &GenericGraph<T1, A1>
    {
        &self.graph_1
    }

    pub fn graph_2(&self) -> &GenericGraph<T2, A2>
    {
        &self.graph_2
    }

    pub fn graph_1_mut(&mut self) -> &mut GenericGraph<T1, A1>
    {
        &mut self.graph_1
    }

    pub fn graph_2_mut(&mut self) -> &mut GenericGraph<T2, A2>
    {
        &mut self.graph_2
    }

    pub fn adj_1(&self) -> &[ADJ]
    {
        &self.adj_1
    }

    pub fn adj_2(&self) -> &[ADJ]
    {
        &self.adj_2
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

impl<ADJ, T1, A1, T2, A2> DualGraph<ADJ, T1, A1, T2, A2>
where A1: AdjContainer<T1>,
    ADJ: AdjTrait
{
    pub fn degree_1(&self, index: usize) -> usize
    {
        self.graph_1.container(index)
            .degree() + self.adj_1[index].slice().len()
    }
}

impl<ADJ, T1, A1, T2, A2> DualGraph<ADJ, T1, A1, T2, A2>
where A2: AdjContainer<T2>,
    ADJ: AdjTrait
{
    pub fn degree_2(&self, index: usize) -> usize
    {
        self.graph_2.container(index)
            .degree() + self.adj_2[index].slice().len()
    }
}

impl<T1, T2, A1, A2, ADJ> DualGraph<ADJ, T1, A1, T2, A2>
where A1: AdjContainer<T1>,
    A2: AdjContainer<T2>,
    ADJ: AdjTrait
{
    /// Depth first search iterator starting at the node corresponding to `index`
    /// 
    /// Note that, this iterator will return indices from both graphs, the corresponding
    /// graph is given by the variant of DualIndex
    pub fn dfs_index(&self, index: DualIndex) -> impl '_ + Iterator<Item=DualIndex>
    {
        DfsDualIndex::new(self, index)
    }

    /// Depth first search iterator starting at the node in graph_1 corresponding to `index`
    /// 
    /// Note that, this iterator will return indices from both graphs, the corresponding
    /// graph is given by the variant of DualIndex
    pub fn dfs_1_index(&self, index: usize) -> impl '_ + Iterator<Item=DualIndex>
    {
        self.dfs_index(DualIndex::Graph1(index))
    }

    /// Depth first search iterator starting at the node in graph_2 corresponding to `index`
    /// 
    /// Note that, this iterator will return indices from both graphs, the corresponding
    /// graph is given by the variant of DualIndex
    pub fn dfs_2_index(&self, index: usize) -> impl '_ + Iterator<Item=DualIndex>
    {
        self.dfs_index(DualIndex::Graph2(index))
    }

    pub fn bfs_index(&self, index: DualIndex) -> impl '_ + Iterator<Item=(DualIndex, usize)>
    {
        BfsDualIndex::new(self, index)
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

    /// * returns `None` **if** graph not connected **or** does not contain any vertices
    /// * uses repeated breadth first search
    pub fn diameter(&self) -> Option<usize>
    {
        if self.total_vertices() == 0 || !self.is_connected(){
            // This could be further optimized by using the first consumtion of the 
            // bfs Iterator to also check if the network is connected,
            // however I did not bother to
            None
        } else {
            let mut max_depth = 0;
            // only one of the vertex counts can be 0, otherwise the total vertex count would be zero
            // also, we have a connected graph
            // Though, which one i choose here does not matter, as I reuse the Iterator before its first use anyway 
            let mut bfs = BfsDualIndex::new(self, DualIndex::Graph1(0));


            for i in 0..self.graph_1.vertex_count() {
                bfs.reuse(DualIndex::Graph1(i));
                let consumable = &mut bfs;
                let depth = match consumable.last(){
                    Some((.., depth)) => depth,
                    None => {
                        // safety: The Iterator will return at least one element as it was "reused"
                        // with a valid index. Therefore "last" cannot return None
                        #[cfg(debug_assertions)]
                        {
                            unreachable!()
                        }
                        #[cfg(not(debug_assertions))]
                        unsafe{
                            std::hint::unreachable_unchecked()
                        }
                    }
                };
                max_depth = max_depth.max(depth);
            }
            // I can ignore one node here, as the last iteration would be redundant
            for i in 1..self.graph_2.vertex_count() {
                bfs.reuse(DualIndex::Graph2(i));
                let consumable = &mut bfs;
                let depth = match consumable.last() {
                    Some((.., depth)) => depth,
                    None => {
                        // safety: The Iterator will return at least one element as it was "reused"
                        // with a valid index. Therefore "last" cannot return None
                        #[cfg(debug_assertions)]
                        {
                            unreachable!()
                        }
                        #[cfg(not(debug_assertions))]
                        unsafe{
                            std::hint::unreachable_unchecked()
                        }
                    }
                };
                max_depth = max_depth.max(depth);
            }
            Some(max_depth)
        }
    }
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
/// Stores information T and the graph the information corresponds to
pub enum WhichGraph<T>{
    /// information is related to graph 1
    Graph1(T),
    /// information is related to graph 2
    Graph2(T)
}

impl<T> Deref for WhichGraph<T>
{
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        match self{
            WhichGraph::Graph1(inner) => inner,
            WhichGraph::Graph2(inner) => inner,
        }
    }
}

impl<T> DerefMut for WhichGraph<T>
{
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        match self{
            WhichGraph::Graph1(inner) => inner,
            WhichGraph::Graph2(inner) => inner
        }
    }
}

impl<T> WhichGraph<T>
{
    #[inline(always)]
    pub fn into_inner(self) -> T
    {
        match self{
            WhichGraph::Graph1(inner) => inner,
            WhichGraph::Graph2(inner) => inner
        }
    }
}

impl<T, ADJ, A1, A2> DualGraph<ADJ, T, A1, T, A2>
where A1: AdjContainer<T> + AdjList<usize>,
    A2: AdjContainer<T> + AdjList<usize>,
    ADJ: AdjTrait,
{
    #[inline]
    pub fn graph_1_contained_iter_mut_which_graph_with_index(&mut self, index: usize) -> impl Iterator<Item=WhichGraph<(usize, &mut T)>>
    {
        assert!(
            index < self.graph_1().vertices.len(),
            "graph_1_contained_iter_mut_which_graph - index out of bounds"
        );

        let ptr = self.graph_1.vertices.as_mut_ptr();
        let iter_helper: &mut A1 = unsafe { &mut *ptr.add(index) };
        let iter = iter_helper.edges();

        let iter = NContainedIterMut::new(
            &mut self.graph_1.vertices,
            iter.iter()
        );

        let o = {
            let slice = self.adj_1[index].slice();
            if slice.is_empty(){
                None
            } else {
                let index = slice[0];
                Some(WhichGraph::Graph2((index, self.graph_2.at_mut(index))))
            }
        };

        iter
            .enumerate()
            .map(WhichGraph::Graph1)
            .chain(o.into_iter())
    }

    #[inline]
    pub fn graph_1_contained_iter_mut_which_graph(&mut self, index: usize) -> impl Iterator<Item=WhichGraph<&mut T>>
    {
        assert!(
            index < self.graph_1().vertices.len(),
            "graph_1_contained_iter_mut_which_graph - index out of bounds"
        );

        let ptr = self.graph_1.vertices.as_mut_ptr();
        let iter_helper: &mut A1 = unsafe { &mut *ptr.add(index) };
        let iter = iter_helper.edges();

        let iter = NContainedIterMut::new(
            &mut self.graph_1.vertices,
            iter.iter()
        );

        let o = {
            let slice = self.adj_1[index].slice();
            if slice.is_empty(){
                None
            } else {
                Some(WhichGraph::Graph2(self.graph_2.at_mut(slice[0])))
            }
        };

        iter
            .map(WhichGraph::Graph1)
            .chain(o.into_iter())
    }


    #[inline]
    pub fn graph_2_contained_iter_mut_which_graph(&mut self, index: usize) -> impl Iterator<Item=WhichGraph<&mut T>>
    {
        assert!(
            index < self.graph_2().vertices.len(),
            "graph_2_contained_iter_mut_which_graph - index out of bounds"
        );

        let ptr = self.graph_2.vertices.as_mut_ptr();
        let iter_helper: &mut A2 = unsafe { &mut *ptr.add(index) };
        let iter = iter_helper.edges();

        let iter = NContainedIterMut::new(
            &mut self.graph_2.vertices,
            iter.iter()
        );

        let o = {
            let slice = self.adj_2[index].slice();
            if slice.is_empty(){
                None
            } else {
                Some(WhichGraph::Graph1(self.graph_1.at_mut(slice[0])))
            }
        };

        iter
            .map(WhichGraph::Graph2)
            .chain(o.into_iter())
    }

    #[inline]
    pub fn graph_2_contained_iter_mut_which_graph_with_index(&mut self, index: usize)-> impl Iterator<Item=WhichGraph<(usize, &mut T)>>
    {
        assert!(
            index < self.graph_2().vertices.len(),
            "graph_2_contained_iter_mut_which_graph - index out of bounds"
        );

        let ptr = self.graph_2.vertices.as_mut_ptr();
        let iter_helper: &mut A2 = unsafe { &mut *ptr.add(index) };
        let iter = iter_helper.edges();

        let iter = NContainedIterMut::new(
            &mut self.graph_2.vertices,
            iter.iter()
        );

        let o = {
            let slice = self.adj_2[index].slice();
            if slice.is_empty(){
                None
            } else {
                let index = slice[0];
                Some(WhichGraph::Graph1((index, self.graph_1.at_mut(index))))
            }
        };


        iter
            .enumerate()
            .map(WhichGraph::Graph2)
            .chain(o.into_iter())
    }

}

//#[inline(always)]
//fn map_chain<'a, T, A, I, F>(
//    ncontained: NContainedIterMut<'a, T, A, I>, 
//    option: Option<WhichGraph<&'a mut T>>,
//    graph_fun: F
//) -> impl Iterator<Item=WhichGraph<&mut T>>
//where I: Iterator<Item=&'a usize>,
//    A: AdjContainer<T>,
//    F: Fn(&'a mut T) -> WhichGraph<&'a mut T>
//{
//    ncontained
//            .map(graph_fun)
//            .chain(option.into_iter())
//}

impl<T, A1, A2, ADJ> DualGraph<ADJ, T, A1, T, A2>
where A1: AdjContainer<T>,
    A2: AdjContainer<T>,
    ADJ: AdjTrait,
{
    /// Iterate over all neighbors of the node corresponding to
    /// index from graph_1. 
    /// Note, that this also includes possible neighbors in the second 
    /// graph
    pub fn contained_iter_neighbors_1(&self, index: usize) -> impl Iterator<Item=&T>
    {
        self.graph_1
            .contained_iter_neighbors(index)
            .chain(
                NContainedIter2::new(
                    self.graph_2.vertices.as_slice(), 
                    self.adj_1[index].slice()
                )
            )
    }

    /// Iterate over all neighbors of the node corresponding to
    /// index from graph_1. 
    /// Note, that this also includes possible neighbors in the second 
    /// graph
    pub fn contained_iter_neighbors_2(&self, index: usize) -> impl Iterator<Item=&T>
    {
        self.graph_2
            .contained_iter_neighbors(index)
            .chain(
                NContainedIter2::new(
                    self.graph_1.vertices.as_slice(), 
                    self.adj_2[index].slice()
                )
            )
    }

    pub fn dfs_contained(&self, index: DualIndex) -> DfsDualContained<T, A1, A2, ADJ>
    {
        DfsDualContained::new(self, index)
    }

    pub fn dfs_1_contained(&self, index_graph_1: usize) -> DfsDualContained<T, A1, A2, ADJ>
    {
        DfsDualContained::new(self, DualIndex::Graph1(index_graph_1))
    }

    pub fn dfs_2_contained(&self, index_graph_2: usize) -> DfsDualContained<T, A1, A2, ADJ>
    {
        DfsDualContained::new(self, DualIndex::Graph2(index_graph_2))
    }
}

pub struct Adj{
    pub(crate) adj: Vec<usize>
}

impl AdjTrait for Adj
{
    fn new() -> Self
    {
        Self{
            adj: Vec::new()
        }
    }

    fn is_adjacent(&self, other_index: &usize) -> bool
    {
        self.adj.contains(other_index)
    }

    fn add_edge(&mut self, other_index: usize) -> bool
    {
        if self.is_adjacent(&other_index){
            return false;
        }
        self.adj.push(other_index);
        true
    }

    fn slice(&self) -> &[usize]
    {
        self.adj.as_slice()
    }
}

pub trait AdjTrait{
    fn new() -> Self;

    fn is_adjacent(&self, other_index: &usize) -> bool;

    fn add_edge(&mut self, other_index: usize) -> bool;

    fn slice(&self) -> &[usize];
}

#[derive(Clone, Copy)]
pub enum AdjSingle
{
    Nothing([usize;0]),
    Something([usize;1])
}

impl AdjSingle
{
    pub fn is_nothing(&self) -> bool
    {
        matches!(self, Self::Nothing(..))
    }

    pub fn is_something(&self) -> bool{
        matches!(self, Self::Something(..))
    }
}

impl AdjTrait for AdjSingle
{
    #[inline(always)]
    fn new() -> Self
    {
        Self::Nothing([])
    }

    #[inline(always)]
    fn is_adjacent(&self, other_index: &usize) -> bool
    {
        if let Self::Something(s) = self
        {
            s[0] == *other_index
        } else {
            false
        }
    }

    #[inline(always)]
    fn add_edge(&mut self, other_index: usize) -> bool
    {
        if self.is_nothing(){
            *self = Self::Something([other_index]);
            true
        } else {
            false
        }
    }

    #[inline(always)]
    fn slice(&self) -> &[usize]
    {
        match self
        {
            Self::Nothing(nothing) => nothing,
            Self::Something(something) => something
        }
    }
}


/// Index which also stores for which graph the index is
pub type DualIndex = WhichGraph<usize>;

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

        let mut dual = SingleDualGraph::new(graph_1, graph_2);

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

        let mut dual = MultiDualGraph::new(
            graph_1,
            graph_2
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

    #[test]
    fn dual_graph_bfs_test()
    {
        let is_connected = |dual_graph: &MultiDualGraph<_, _, _, _>| {
            dual_graph.bfs_index(DualIndex::Graph1(0)).count() == dual_graph.total_vertices()
        };

        
        let mut graph_1 = Graph::<EmptyNode>::new(5);
        graph_1.add_edge(0, 1).unwrap();
        graph_1.add_edge(1, 2).unwrap();
        graph_1.add_edge(3, 4).unwrap();

        assert_eq!(graph_1.is_connected(), Some(false));

        let mut graph_2 = SwGraph::<CountingNode>::new(5);

        graph_2.init_ring_2();
        assert_eq!(graph_2.is_connected(), Some(true));

        let mut dual = DualGraph::new(graph_1, graph_2);
        
        assert!(!is_connected(&dual));
        is_connected(&dual);

        dual.add_edge(1, 0).unwrap();
        assert!(!is_connected(&dual));

        dual.add_edge(3, 4).unwrap();
        // now it should be connected
        assert!(is_connected(&dual));

    }

    #[test]
    fn dual_graph_bfs_depth_test()
    {
        let mut graph_1 = Graph::<EmptyNode>::new(6);
        graph_1.init_ring(1).unwrap();
        let graph_2 = graph_1.clone();

        let mut dual = MultiDualGraph::new(graph_1, graph_2);

        // connect them
        dual.add_edge(0, 0).unwrap();

        let depth = dual.bfs_index(DualIndex::Graph1(3)).last().unwrap().1;
        assert!(dual.is_connected());
        assert_eq!(depth, 7);

        assert_eq!(dual.diameter(), Some(7));

    }
}