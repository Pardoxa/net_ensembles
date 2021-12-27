use {
    crate::{GenericGraph, AdjContainer},
    super::dual_graph_iterators::*
};

pub struct DualGraph<'a, T1, A1, T2, A2>
{
    pub(crate) graph_1: &'a GenericGraph<T1, A1>,
    pub(crate) graph_2: &'a GenericGraph<T2, A2>,
    pub(crate) adj_1: Vec<Adj>,
    pub(crate) adj_2: Vec<Adj>
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

    pub fn bfs_index(&'a self, index: DualIndex) -> impl 'a + Iterator<Item=(DualIndex, usize)>
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

pub(crate) struct Adj{
    pub(crate) adj: Vec<usize>
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

/// Index which also stores for which graph the index is
#[derive(Debug, Clone)]
pub enum DualIndex
{
    /// Index for graph_1
    Graph1(usize),
    /// Index for graph_2
    Graph2(usize)
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

    #[test]
    fn dual_graph_bfs_test()
    {
        let is_connected = |dual_graph: &DualGraph<_, _, _, _>| {
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

        let mut dual = DualGraph::new(&graph_1, &graph_2);
        
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

        let mut dual = DualGraph::new(&graph_1, &graph_2);

        // connect them
        dual.add_edge(0, 0).unwrap();

        let depth = dual.bfs_index(DualIndex::Graph1(3)).last().unwrap().1;
        assert!(dual.is_connected());
        assert_eq!(depth, 7);

        assert_eq!(dual.diameter(), Some(7));

    }
}