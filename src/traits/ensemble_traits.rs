use crate::{
    AdjContainer,
    traits::*, 
    iter::*, 
    GenericGraph,
    sw_graph::SwContainer,
    graph::NodeContainer,
    generic_graph::{Dfs, DfsWithIndex, Bfs}
};

/// unify graph ensembles in a trait
pub trait WithGraph<T, G> {
    /// * access additional information at index
    fn at(&self, index: usize) -> &T;

    /// * mutable access to additional information at index
    fn at_mut(&mut self, index: usize) -> &mut T;

    /// * returns reference to the underlying topology aka, the `GenericGraph`
    /// * use this to call functions regarding the topology
    fn graph(&self) -> &G;

    /// * sorts Adjecaency List
    fn sort_adj(&mut self);
}

///  Collection mut Graph iterators
pub trait GraphIteratorsMut<T, G, A>
where
    T: Node,
    A: AdjContainer<T>
{
    /// * iterate over mutable additional information of neighbors of vertex `index`
    /// * iterator returns `&mut T`
    /// * `sort_adj` will affect the order
    /// * **panics** if index out of bounds
    fn contained_iter_neighbors_mut(&mut self, index: usize) -> NContainedIterMut<'_, T, A, IterWrapper>;

    /// * iterate over mutable additional information of neighbors of vertex `index`
    /// * iterator returns `(index_neighbor: usize, neighbor: &mut T)`
    /// * `sort_adj` will affect the order
    /// * **panics** if index out of bounds
    fn contained_iter_neighbors_mut_with_index(&mut self, index: usize) -> INContainedIterMut<'_, T, A>;

    /// * get iterator over mutable additional information stored at each vertex in order of the indices
    /// * iterator returns a `Node` (for example `EmptyNode` or whatever you used)
    fn contained_iter_mut(&mut self) -> ContainedIterMut<'_, T, A>;
}

/// Collection of Graph iterators
pub trait GraphIterators<T, G, A>
    where
        T: Node,
        A: AdjContainer<T>
{
    /// * get iterator over additional information stored at each vertex in order of the indices
    /// * iterator returns a `Node` (for example `EmptyNode` or whatever you used)
    /// * similar to `self.container_iter().map(|container| container.contained())`
    fn contained_iter(&self) -> ContainedIter<'_, T, A>;

    /// * iterate over additional information of neighbors of vertex `index`
    /// * iterator returns `&T`
    /// * `sort_adj` will affect the order
    /// * **panics** if index out of bounds
    fn contained_iter_neighbors(&self, index: usize) -> NContainedIter<'_, T, A>;

    /// * iterate over additional information of neighbors of vertex `index`
    /// * iterator returns (`index_neighbor`,`&T`)
    /// * `sort_adj` will affect the order
    /// * **panics** if index out of bounds
    fn contained_iter_neighbors_with_index(&self, index: usize) -> NIContainedIter<T, A>;

    /// * get iterator over AdjContainer in order of the indices
    /// * iterator returns `AdjContainer<Node>`, i.e., `A`
    fn container_iter(&self) -> core::slice::Iter<'_, A>;

    /// * iterate over additional information of neighbors of vertex `index`
    /// * iterator returns `&T`
    /// * `sort_adj` will affect the order
    /// * **panics** if index out of bounds
    fn container_iter_neighbors(&self, index: usize) -> NContainerIter<'_, T, A>;

    /// # returns `Iterator`
    ///
    /// * the iterator will iterate over the vertices in depth first search order,
    /// beginning with vertex `index`.
    /// * iterator returns `node`
    ///
    /// Order
    ///------------------------
    /// Order is guaranteed to be in DFS order, however
    /// if this order is not unambigouse
    /// adding edges and especially removing edges will shuffle the order.
    ///
    /// Note:
    /// ----------------------
    /// Will only iterate over vertices within the connected component that contains vertex `index`
    fn dfs(&self, index: usize) -> Dfs<'_, T, A>;

    /// # returns `Iterator`
    ///
    /// * the iterator will iterate over the vertices in depth first search order,
    /// beginning with vertex `index`.
    /// * Iterator returns tuple `(index, node)`
    ///
    /// Order
    ///------------------------
    /// Order is guaranteed to be in DFS order, however
    /// if this order is not unambigouse
    /// adding edges and especially removing edges will shuffle the order.
    ///
    /// Note:
    /// ----------------------
    /// Will only iterate over vertices within the connected component that contains vertex `index`
    fn dfs_with_index(&self, index: usize) -> DfsWithIndex<'_, T, A>;

    /// # returns `Iterator`
    ///
    /// * the iterator will iterate over the vertices in breadth first search order,
    /// beginning with vertex `index`.
    /// * Iterator returns tuple `(index, node, depth)`
    ///
    /// ### depth
    /// * starts at 0 (i.e. the first element in the iterator will have `depth = 0`)
    /// * `depth` equals number of edges in the *shortest path* from the *current* vertex to the
    /// *first* vertex (i.e. to the vertex with index `index`)
    ///
    /// Order
    ///------------------------
    /// Order is guaranteed to be in BFS order, however
    /// if this order is not unambigouse
    /// adding edges and especially removing edges will shuffle the order.
    ///
    /// Note:
    /// ----------------------
    /// Will only iterate over vertices within the connected component that contains vertex `index`
    fn bfs_index_depth(&self, index: usize) -> Bfs<'_, T, A>;
}


impl<T, E> GraphIterators<T, GenericGraph<T, NodeContainer<T>>, NodeContainer<T>> for E
where
    T: Node,
    E: WithGraph<T, GenericGraph<T, NodeContainer<T>>>,
{
    fn contained_iter(&self) -> ContainedIter<'_, T, NodeContainer<T>>
    {
        self.graph().contained_iter()
    }

    fn contained_iter_neighbors(&self, index: usize) -> NContainedIter<'_, T, NodeContainer<T>>
    {
        self.graph().contained_iter_neighbors(index)
    }

    fn container_iter(&self) -> core::slice::Iter<'_, NodeContainer<T>>
    {
        self.graph().container_iter()
    }

    fn container_iter_neighbors(&self, index: usize) -> NContainerIter<'_, T, NodeContainer<T>>
    {
        self.graph().container_iter_neighbors(index)
    }

    fn contained_iter_neighbors_with_index(&self, index: usize) -> NIContainedIter<T, NodeContainer<T>> {
        self.graph().contained_iter_neighbors_with_index(index)
    }

    fn dfs(&self, index: usize) -> Dfs<'_, T, NodeContainer<T>>
    {
        self.graph().dfs(index)
    }

    fn dfs_with_index(&self, index: usize) -> DfsWithIndex<'_, T, NodeContainer<T>>
    {
        self.graph().dfs_with_index(index)
    }

    fn bfs_index_depth(&self, index: usize) -> Bfs<'_, T, NodeContainer<T>>
    {
        self.graph().bfs_index_depth(index)
    }
}

impl<T, E> GraphIterators<T, GenericGraph<T, SwContainer<T>>, SwContainer<T>> for E
where
    T: Node,
    E: WithGraph<T, GenericGraph<T, SwContainer<T>>>,
{
    fn contained_iter(&self) -> ContainedIter<'_, T, SwContainer<T>>
    {
        self.graph().contained_iter()
    }

    fn contained_iter_neighbors(&self, index: usize) -> NContainedIter<'_, T, SwContainer<T>>
    {
        self.graph().contained_iter_neighbors(index)
    }

    fn container_iter(&self) -> core::slice::Iter<'_, SwContainer<T>>
    {
        self.graph().container_iter()
    }

    fn container_iter_neighbors(&self, index: usize) -> NContainerIter<'_, T, SwContainer<T>>
    {
        self.graph().container_iter_neighbors(index)
    }

    fn contained_iter_neighbors_with_index(&self, index: usize)
        -> NIContainedIter<T, SwContainer<T>>
    {
        self.graph().contained_iter_neighbors_with_index(index)
    }

    fn dfs(&self, index: usize) -> Dfs<'_, T, SwContainer<T>>
    {
        self.graph().dfs(index)
    }

    fn dfs_with_index(&self, index: usize) -> DfsWithIndex<'_, T, SwContainer<T>>
    {
        self.graph().dfs_with_index(index)
    }

    fn bfs_index_depth(&self, index: usize) -> Bfs<'_, T, SwContainer<T>>
    {
        self.graph().bfs_index_depth(index)
    }
}
