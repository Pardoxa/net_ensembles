use{
    super::iterators::*,
    crate::{
        traits::*, 
        iter::*, 
        GraphErrors,
        graph::{
            NodeContainer,
            Graph
        }
    },
    sampling::histogram::{
        HistUsizeFast,
        HistogramVal
    },
    std::{
        convert::*,
        collections::*,
        marker::*,
        io::Write
    },
    rand::Rng
};

use std::num::NonZeroUsize;

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};
/// # Generic graph implementation
/// * contains multiple measurable quantities
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct GenericGraph<T, A>
{
    pub(crate) next_id: usize,
    pub(crate) edge_count: usize,
    pub(crate) vertices: Vec<A>,
    pub(crate) phantom: PhantomData<T>,
}

impl<T, A> GenericGraph<T, A>
where T: Node,
      A: AdjContainer<T> {
    /// Create new graph with `size` nodes
    /// and no edges
    pub fn new(size: usize) -> Self {
        let mut vertices = Vec::with_capacity(size);
        vertices.extend(
            (0..size)
                .map(|i| A::new(i, T::new_from_index(i)))
        );
        Self{
            vertices,
            next_id: size,
            edge_count: 0,
            phantom: PhantomData,
        }
    }

    /// Use the topology from another graph and create a new one with 
    /// the same topology but the function 'map' is used to 
    /// map what is contained in the original Graph to the newly created one.
    pub fn clone_topology<F, T2>(&self, mut map: F) -> GenericGraph<T2, NodeContainer<T2>>
    where F: FnMut (&T) -> T2
    {
        let nodes: Vec<_> = self.vertices
            .iter()
            .enumerate()
            .map(
            |(index,node)|
            {
                let neighbors = node.neighbors().into_vec();
                NodeContainer{
                    id: index,
                    adj: neighbors,
                    node: map(node.contained())

                }
            }
        ).collect();
        GenericGraph{
            next_id: nodes.len(),
            edge_count: self.edge_count,
            vertices: nodes,
            phantom: PhantomData
        }
    }
}

impl<T, A> GenericGraph<T, A>
{
    /// Get internal vertice slice
    pub fn get_vertices(&self) -> &[A]
    {
        &self.vertices
    }
}

impl<T, A> GenericGraph<T, A>
where A: AdjContainer<T>
{
    /// # create a new graph
    /// * graph will contain `contained.len()` vertices, which will contain the corresponding entries
    /// of the vector `contained`
    /// * graph will not contain any edges upon creation
    /// ```
    /// use net_ensembles::{GenericGraph, graph::NodeContainer};
    /// 
    /// let graph = GenericGraph::<_, NodeContainer::<_>>::from_vec(vec!["first", "second", "third"]);
    /// assert_eq!(graph.at(0), &"first");
    /// assert_eq!(graph.at(1), &"second");
    /// assert_eq!(graph.at(2), &"third");
    /// assert_eq!(graph.vertex_count(), 3);
    /// assert_eq!(graph.edge_count(), 0);
    /// ```
    pub fn from_vec(contained: Vec<T>) -> Self
    {
        let vertices: Vec<_> = contained.into_iter()
            .enumerate()
            .map(|(index, t)| A::new(index, t))
            .collect();
        
        Self{
            next_id: vertices.len(),
            vertices,
            edge_count: 0,
            phantom: PhantomData
        }
    }
}

impl<T, A> GenericGraph<T, A>
where A: AdjContainer<T>
{

    /// # removes all edges from the graph
    /// * inexpensive O(1), if there are no edges to begin with
    /// * O(vertices) otherwise
    pub fn clear_edges(&mut self) {
        if self.edge_count() != 0 {
            self.edge_count = 0;
            for container in self.vertices.iter_mut() {
                unsafe { container.clear_edges(); }
            }
        }
    }

    /// # initialize Ring
    /// * every node is connected with its neighbors, which are
    /// not more than distance away
    pub(crate) fn init_ring(&mut self, distance: NonZeroUsize) -> Result<(), GraphErrors>
    {
        self.clear_edges();
        let n = self.vertex_count();

        for i in 0..n
        {
            for add in 1..=distance.get(){
                let sum = i + add;
                let index2 = if sum >= n {
                    sum - n
                } else {
                    sum
                };
                self.add_edge(i, index2)?
            }
        }
        Ok(())
    }

    /// # Sort adjecency lists
    /// If you depend on the order of the adjecency lists, you can sort them
    /// # Performance
    /// * internally uses [pattern-defeating quicksort](https://github.com/orlp/pdqsort)
    /// as long as that is the standard
    /// * sorts an adjecency list with length `d` in worst-case: `O(d log(d))`
    /// * is called for each adjecency list, i.e., `self.vertex_count()` times
    pub fn sort_adj(&mut self) {
        for container in self.vertices.iter_mut() {
            container.sort_adj();
        }
    }

    /// Shuffles all adjacency lists
    /// * This will not change the topology. It will change the internal order and thereby 
    /// randomize the order of the neighbors in the respective iterators
    pub fn shuffle_adjs<R: Rng>(&mut self, rng: &mut R)
    {
        self.vertices.iter_mut()
            .for_each(
                |v|
                v.shuffle_adj(rng)
            );
    }

    /// # Shuffles adjacency list
    /// * panics if index is out of bounds
    /// * This will not change the topology. It will change the internal order and thereby 
    /// randomize the order of the neighbors in the respective iterators
    pub fn shuffle_adj<R: Rng>(&mut self, rng: &mut R, index: usize)
    {
        self.vertices[index].shuffle_adj(rng)
    }


    /// # get `AdjContainer` of vertex `index`
    /// * **panics** if index out of bounds
    pub fn container(&self, index: usize) -> &A {
        &self.vertices[index]
    }

    /// # get `AdjContainer` of vertex `index`
    /// * `None` if index is out of bounds
    /// * `Some(&A)` else
    pub fn container_checked(&self, index: usize) -> Option<&A>
    {
        self.vertices.get(index)
    }

    /// * get iterator over AdjContainer in order of the indices
    /// * iterator returns `AdjContainer<Node>`
    pub fn container_iter(&self) -> std::slice::Iter::<A> {
        self.vertices.iter()
    }

    /// * iterate over `AdjContainer` of neighbors of vertex `index`
    /// * iterator returns `AdjContainer<Node>`
    /// * `sort_adj` will affect the order
    ///
    ///   If `let mut iter = self.contained_iter_neighbors()` is called directly after
    ///   `self.sort_adj()`, the following will be true (as long as `iter` does not return `None` of cause):
    ///   `iter.next().unwrap().id() < iter.next().unwrap.id() < ...` Note, that `...id()` returns the
    ///   index of the corresponding vertex
    /// * **panics** if index out of bounds
    pub fn container_iter_neighbors(&self, index: usize) -> NContainerIter<T, A, IterWrapper> {
        NContainerIter::new(
            self.vertices.as_slice(),
            self.vertices[index].neighbors()
        )
    }

    /// * get iterator over additional information stored at each vertex in order of the indices
    /// * iterator returns a `Node` (for example `EmptyNode` or whatever you used)
    /// * similar to `self.container_iter().map(|container| container.contained())`
    pub fn contained_iter(&self) -> ContainedIter<T, A> {
        ContainedIter::new(self.vertices.as_slice())
    }

    /// * same as `contained_iter`, but mutable
    pub fn contained_iter_mut(&mut self) -> ContainedIterMut<T, A> {
        ContainedIterMut::new (
            self.vertices.iter_mut()
        )

    }

    /// * iterate over additional information of neighbors of vertex `index`
    /// * iterator returns `&T`
    /// * `sort_adj` will affect the order
    /// * **panics** if index out of bounds
    pub fn contained_iter_neighbors(&self, index: usize) -> NContainedIter<T, A, IterWrapper> {
        NContainedIter::new(
            self.vertices.as_slice(),
            self.vertices[index].neighbors()
        )
    }

    /// * iterate over additional information of neighbors of vertex `index`
    /// * iterator returns (`index_neighbor`,`&T`)
    /// * `sort_adj` will affect the order
    /// * **panics** if index out of bounds
    pub fn contained_iter_neighbors_with_index(&self, index: usize) -> NIContainedIter<T, A> {
        NIContainedIter::new(
            self.vertices.as_slice(),
            self.vertices[index].neighbors()
        )
    }

    /// * iterate over mutable additional information of neighbors of vertex `index`
    /// * iterator returns `&mut T`
    /// * `sort_adj` will affect the order
    /// * **panics** if index out of bounds
    /// * See also: [`GraphIteratorsMut`](../traits/trait.GraphIteratorsMut.html)
    pub fn contained_iter_neighbors_mut(&mut self, index: usize) -> NContainedIterMut<T, A, IterWrapper> {
        assert!(
            index < self.vertices.len(),
            "contained_iter_neighbors_mut - index out of bounds"
        );

        let ptr = self.vertices.as_mut_ptr();
        let iter_helper: &mut A = unsafe { &mut *ptr.add(index) };
        let iter = iter_helper.neighbors();

        NContainedIterMut::new(
            self.vertices.as_mut_slice(),
            iter
        )
    }

    /// * iterate over mutable additional information of neighbors of vertex `index`
    /// * iterator returns `(index_neighbor: usize, neighbor: &mut T)`
    /// * `sort_adj` will affect the order
    /// * **panics** if index out of bounds
    /// * See also: [`GraphIteratorsMut`](../traits/trait.GraphIteratorsMut.html)
    pub fn contained_iter_neighbors_mut_with_index(&mut self, index: usize) -> INContainedIterMut<T, A> {
        assert!(
            index < self.vertices.len(),
            "contained_iter_neighbors_mut_with_index - index out of bounds"
        );

        let ptr = self.vertices.as_mut_ptr();
        let iter_helper: &mut A = unsafe { &mut *ptr.add(index) };
        let iter = iter_helper.neighbors();

        INContainedIterMut::new(
            self.vertices.as_mut_slice(),
            iter
        )
    }

    pub(crate) fn container_mut(&mut self, index: usize) -> &mut A {
        &mut self.vertices[index]
    }

    /// # For your calculations etc.
    /// * **read access** to **your struct** T, stored at **each vertex**, that implements `Node` trait
    /// ## Note
    /// * **panics** if index out of bounds
    pub fn at(&self, index: usize) -> &T {
        self.container(index).contained()
    }

    /// # For your calculations etc.
    /// * **read access** to **your struct** T, stored at **each vertex**, that implements `Node` trait
    /// * `None` if index is out of bounds
    pub fn at_checked(&self, index: usize) -> Option<&T>
    {
        self.container_checked(index)
            .map(|container| container.contained())
    }

    /// # For your calculations etc.
    /// * **write access** to **your struct** T, stored at **each vertex**, that implements `Node` trait
    /// # Note    
    /// * **panics** if index out of bounds
    pub fn at_mut(&mut self, index: usize) -> &mut T {
        self.container_mut(index).contained_mut()
    }

    /// returns number of vertices present in graph
    pub fn vertex_count(&self) -> usize {
        self.next_id
    }

    /// calculates the average degree of the graph
    /// * `(2 * edge_count) / vertex_count`
    pub fn average_degree(&self) -> f32 {
        (2 * self.edge_count()) as f32 / self.vertex_count() as f32
    }

    /// # Get mutable vertex
    /// * panics if index out of range
    pub(crate) fn get_mut_unchecked(&mut self, index: usize) -> &mut A {
        &mut self.vertices[index]
    }

    /// Returns two mutable references in a tuple
    /// ## panics in debug if:
    /// * index out of bounds
    /// * if indices are not unique
    pub(crate) fn get_2_mut(&mut self, index0: usize, index1: usize) -> (&mut A, &mut A)
    {
        debug_assert!(
            index0 < self.next_id &&
            index1 < self.next_id,
            "net_ensembles - panic - index out of bounds! \
                vertex_count: {}, index_0: {}, index1: {} - \
                error probably results from trying to add or remove edges",
            self.vertex_count(),
            index0,
            index1
            
        );

        debug_assert!(
            index0 != index1,
            "net_ensembles - panic - indices have to be unique! \
            error probably results from trying to add or remove self-loops"
        );

        let r1: &mut A;
        let r2: &mut A;

        let ptr = self.vertices.as_mut_ptr();

        unsafe {
            r1 = &mut *ptr.add(index0);
            r2 = &mut *ptr.add(index1);
        }

        (r1, r2)
    }

    /// Returns three mutable references in a tuple
    /// ## panics:
    /// * index out of bounds
    /// * in debug: if indices are not unique
    pub(crate) fn get_3_mut(&mut self, index0: usize, index1: usize, index2: usize) ->
        (&mut A, &mut A, &mut A)
    {
        debug_assert!(
            index0 < self.next_id &&
            index1 < self.next_id &&
            index2 < self.next_id
        );

        debug_assert!(
            index0 != index1 &&
            index1 != index2 &&
            index2 != index0
        );

        let r1: &mut A;
        let r2: &mut A;
        let r3: &mut A;

        let ptr = self.vertices.as_mut_ptr();

        unsafe {
            r1 = &mut *ptr.add(index0);
            r2 = &mut *ptr.add(index1);
            r3 = &mut *ptr.add(index2);
        }

        (r1, r2, r3)
    }

    /// Returns a reference to the element stored in the specified node or `None` if out of Bounds
    pub fn get_contained(&self, index: usize) -> Option<&T>
    {
        self.vertices
            .get(index)
            .map(|v| v.contained())
    }

    /// Returns a mutable reference to the element stored in the specified node or `None` if out of Bounds
    pub fn get_contained_mut(&mut self, index: usize) -> Option<&mut T>
    {
        self.vertices
            .get_mut(index)
            .map(|v| v.contained_mut())
    }

    /// Returns a reference to the element stored in the specified node
    ///
    /// For a save alternative see [get_contained](`Self::get_contained`)
    /// # Safety
    /// Calling this method with an out-of-bounds index is [undefined behavior](https://doc.rust-lang.org/reference/behavior-considered-undefined.html) even if the resulting reference is not used.
    pub unsafe fn get_contained_unchecked(&self, index: usize) -> &T
    {
        self.vertices
            .get_unchecked(index)
            .contained()
    }

    /// Returns a mutable reference to the element stored in the specified node
    ///
    /// For a save alternative see [get_contained_mut](`Self::get_contained_mut`)
    /// # Safety
    /// Calling this method with an out-of-bounds index is [undefined behavior](https://doc.rust-lang.org/reference/behavior-considered-undefined.html) even if the resulting reference is not used.    
    pub unsafe fn get_contained_unchecked_mut(&mut self, index: usize) -> &mut T
    {
        self.vertices
            .get_unchecked_mut(index)
            .contained_mut()
    }

    /// Adds edge between nodes `index1` and `index2`
    /// ## ErrorCases:
    /// | Error | Reason |
    /// | ---- | ---- |
    /// | `GraphErrors::IndexOutOfRange` | `index1` or `index2` larger than `self.vertex_count()`  |
    /// | `GraphErrors::EdgeExists` | requested edge already exists! |
    /// ## panics
    /// * if indices out of bounds
    /// * in debug: If `index0 == index1`
    pub fn add_edge(&mut self, index1: usize, index2: usize) -> Result<(),GraphErrors> {
        let (r1, r2) = self.get_2_mut(index1, index2);
        unsafe{ r1.push(r2)?; }
        self.edge_count += 1;
        Ok(())
    }

    /// Removes edge between nodes *index1* and *index2*
    /// ## ErrorCases:
    /// | Error | Reason |
    /// | ---- | ---- |
    /// | `GraphErrors::IndexOutOfRange` | `index1` or `index2` larger than `self.vertex_count()`  |
    /// | `GraphErrors::EdgeDoesNotExist` | requested edge does not exists |
    /// # panics
    /// * if index out of bounds
    /// * in debug: If `index0 == index1`
    pub fn remove_edge(&mut self, index1: usize, index2: usize) -> Result<(),GraphErrors> {
        let (r1, r2) = self.get_2_mut(index1, index2);
        unsafe{ r1.remove(r2)?; }
        self.edge_count -= 1;
        Ok(())
    }

    /// returns total number of edges in graph
    pub fn edge_count(&self) -> usize {
        self.edge_count
    }

    /// * returns number of vertices adjacent to vertex `index`
    /// * `None` if index out of bounds
    pub fn degree(&self, index: usize) -> Option<usize> {
        Some(
            self
                .vertices
                .get(index)?
                .degree()
        )
    }

    /// # Iterator
    /// Iterate over the degrees of each node (in the order of the indices)
    pub fn degree_iter(&'_ self) -> impl Iterator<Item=usize> + '_
    {
        self.vertices
            .iter()
            .map(A::degree)
    }

    /// # get degree vector
    /// * returns vector of length `self.vertex_count()`
    /// where each entry corresponds to the degree of the vertex with the respective index
    pub fn degree_vec(&self) -> Vec<usize>
    {
        let mut degree_vec = Vec::with_capacity(self.vertex_count());
        degree_vec.extend(self.degree_iter());
        degree_vec
    }

    /// # Get a histogram of the degrees
    /// * You can look at your degree distribution with this
    /// ## Safety
    /// * Assumes you have at least one node, panics otherwise
    pub fn degree_histogram(&self) -> HistUsizeFast
    {
        let mut iter = self.degree_iter();
        let mut min = iter.next()
            .expect("Expected at least one node");

        let mut max = min;

        iter.for_each(
            |val|
            {
                if val < min {
                    min = val
                } else if val > max {
                    max = val
                }
            }
        );

        let mut hist = match HistUsizeFast::new_inclusive(min, max)
        {
            Ok(hist) => hist,
            Err(_) => unreachable!()
        };

        self.degree_iter()
            .for_each(
                |entry|
                {
                    match hist.count_val(entry)
                    {
                        Ok(_) => (),
                        Err(_) => unreachable!(),
                    }
                }
            );

        hist
    }

    /// # returns `Iterator`
    ///
    /// * the iterator will iterate over the vertices in depth first search order,
    /// beginning with vertex `index`.
    /// * iterator returns what is contained at the corresponding vertices, i.e., `&T`
    /// * iterator always returns None if index out of bounds
    ///
    /// Order
    ///------------------------
    /// Order is guaranteed to be in DFS order, however
    /// if this order is not unambiguous
    /// adding edges and especially removing edges will shuffle the order.
    ///
    /// Note:
    /// ----------------------
    /// Will only iterate over vertices within the connected component that contains vertex `index`
    pub fn dfs(&self, index: usize) -> Dfs<T, A> {
        Dfs::new(self, index)
    }

    /// # returns `Iterator`
    ///
    /// * the iterator will iterate over the vertices in depth first search order,
    /// beginning with vertex `index`.
    /// * iterator returns what is contained at the corresponding vertices, i.e., `&mut T`
    /// * iterator always returns None if index out of bounds
    ///
    /// Order
    ///------------------------
    /// Order is guaranteed to be in DFS order, however
    /// if this order is not unambiguous
    /// adding edges and especially removing edges will shuffle the order.
    ///
    /// Note:
    /// ----------------------
    /// Will only iterate over vertices within the connected component that contains vertex `index`
    pub fn dfs_mut(&mut self, index: usize) -> DfsMut<T, A>
    {
        DfsMut::new(self, index)
    }

    /// # returns `Iterator`
    ///
    /// * the iterator will iterate over the vertices in depth first search order,
    /// beginning with vertex `index`.
    /// * Iterator returns tuple `(index, node)`
    /// * iterator always returns None if index out of bounds
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
    pub fn dfs_with_index(&self, index: usize) -> DfsWithIndex<T, A> {
        DfsWithIndex::new(self, index)
    }

    /// # returns `Iterator`
    ///
    /// * the iterator will iterate over the vertices in breadth first search order,
    /// beginning with vertex `index`.
    /// * Iterator returns tuple `(index, node, depth)` where `node` is what is contained at the 
    /// corresponding index, i.e., `&T`
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
    pub fn bfs_index_depth(&self, index: usize) -> Bfs<T, A> {
        Bfs::new(self, index)
    }

    /// # returns `Iterator`
    ///
    /// * the iterator will iterate over the vertices in breadth first search order,
    /// beginning with vertex `index`.
    /// * Iterator returns tuple `(index, node, depth)` where `node` is what is contained at the 
    /// corresponding index, i.e., `&mut T`
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
    pub fn bfs_index_depth_mut(&mut self, index: usize) -> BfsMut<T, A> {
        BfsMut::new(self, index)
    }

    // # returns `Option<Iterator>`
    /// * similar to self.bfs_index_depth, but allows for ignoring of vertices
    ///
    /// * the iterator will iterate over the vertices in breadth first search order,
    /// beginning with vertex `index`.
    /// * the iterator will ignore all vertices, where `filter(vertex, index_of_vertex)` returns false
    /// * returns `None`if index is out of bounds or `filter(vertex, index)`retruns false
    /// * Iterator returns tuple `(index, vertex, depth)`
    ///
    /// ### depth
    /// * starts at 0 (i.e. the first element in the iterator will have `depth = 0`)
    /// * `depth` equals number of edges in the *shortest path* from the *current* vertex to the
    /// *first* vertex (i.e. to the vertex with index `index`), while ignoring paths that 
    /// go through filtered out vertices
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
    pub fn bfs_filtered<F>(&'_ self, index: usize, filter: F) -> Option<BfsFiltered<'_, T, A>>
    where F: FnMut(&T, usize) -> bool,
    {
        BfsFiltered::new(self, index, filter)
    }

    /// | result       |                          condition                       |
    /// |--------------|----------------------------------------------------------|
    /// | `None`       | **if** graph does not contain any vertices               |
    /// | `Some(true)` | **else if** all vertices are connected by paths of edges |
    /// | `Some(false)`| **otherwise**                                            |
    pub fn is_connected(&self) -> Option<bool> {
        if self.vertex_count() == 0 {
            None
        } else {
            Some(self.dfs(0).count() == self.vertex_count())
        }
    }

    /// # definition
    /// Calculates the size of the **q-core** (i.e. number of nodes in the biggest possible set of nodes,
    /// where all nodes from the set are connected with at least `q` other nodes from the set)
    ///
    /// returns `None` if impossible to calculate (e.g. `vertex_count == 0` or `q <= 1`)
    /// # Example
    /// ```
    /// use net_ensembles::EmptyNode;
    /// use net_ensembles::Graph;
    ///
    /// let graph: Graph<EmptyNode> = Graph::new(0);
    /// assert_eq!(graph.q_core(1), None);
    /// assert_eq!(graph.q_core(2), None);
    ///
    /// let graph2: Graph<EmptyNode> = Graph::new(1);
    ///
    /// assert_eq!(graph2.q_core(1), None);
    /// assert_eq!(graph2.q_core(2), Some(0));
    ///
    ///
    /// // create complete graph
    /// let mut graph3: Graph<EmptyNode> = Graph::new(20);
    /// for i in 0..graph3.vertex_count() {
    ///     for j in i+1..graph3.vertex_count() {
    ///         graph3.add_edge(i, j).unwrap();
    ///     }
    /// }
    ///
    /// // since this is a complete graph, the q-core should always consist of 20 nodes
    /// // as long as q < 20, as every node has 19 neighbors
    /// for i in 2..20 {
    ///     assert_eq!(graph3.q_core(i), Some(20));
    /// }
    ///
    /// assert_eq!(graph3.q_core(20), Some(0));
    /// ```
    pub fn q_core(&self, q: usize) -> Option<usize> {
        if q < 2 || self.vertex_count() == 0 {
            return None;
        }

        let mut degree: Vec<_> = self.container_iter()
            .map(|vertex| vertex.degree())
            .collect();

        // virtually: recursively remove all vertices with less then q neighbors
        let mut something_changed = true;

        while something_changed {
            something_changed = false;
            for i in 0..self.vertex_count() {
                if degree[i] == 0 {
                    continue;
                }
                if degree[i] < q {
                    self.vertices[i]
                        .neighbors()
                        .for_each(|&n|
                            {
                                if degree[n] > 0 {
                                    degree[n] -= 1;
                                }
                            }
                        );
                    degree[i] = 0;
                    something_changed = true;
                }
            }
        }

        // find biggest component
        let mut result = 0;
        // initiate stack
        let mut stack: Vec<usize> = Vec::with_capacity(self.vertex_count());

        for i in 0..self.vertex_count() {
            // skip all nodes that are removed or in a known component
            if degree[i] == 0 {
                continue;
            }
            let mut counter = 0;
            stack.push(i);

            // i is in known component
            degree[i] = 0;

            while let Some(index) = stack.pop() {
                counter += 1;

                for &j in self
                    .container(index)
                    .neighbors()    // iterate over neighbors
                {
                    // skip if already handled
                    if degree[j] == 0 {
                        continue;
                    }

                    degree[j] = 0;
                    stack.push(j);
                }
            }
            result = result.max(counter);
        }

        Some(result)
    }

    /// # compute connected component ids
    /// * used for `self.connected_components()`
    /// * each vertex gets an id, all vertices with the same id are in the same connected component
    /// * returns (number of components, vector of ids)
    pub fn connected_components_ids(&self) -> (usize, Vec<isize>)
    {
        let mut component_id : Vec<isize> = vec![-1; self.vertex_count()];
        let mut current_id = 0;

        for i in 0..self.vertex_count(){
            // already in a component?
            if component_id[i] != -1 {
                continue;
            }

            // start depth first search over indices of vertices connected with vertex i
            for (j, _) in self.dfs_with_index(i) {
                component_id[j] = current_id;
            }
            current_id += 1;

        }
        // cast current_id as usize
        let num_components = usize::try_from(current_id)
            .expect("connected_components ERROR 0");

        (num_components, component_id)
    }

    /// # compute sizes of all *connected components*
    ///
    /// * the **number** of connected components is the **size** of the returned vector, i.e. `result.len()`
    /// * returns **empty** vector, if graph does not contain vertices
    /// * returns (reverse) **ordered vector of sizes** of the connected components,
    /// i.e. the biggest component is of size `result[0]` and the smallest is of size `result[result.len() - 1]`
    pub fn connected_components(&self) -> Vec<usize> {

        let (num_components, component_id) = self.connected_components_ids();

        let mut result = vec![0; num_components];

        for i in component_id {
            let as_usize = usize::try_from(i)
                .expect("connected_components ERROR 1");
            result[as_usize] += 1;
        }

        // sort by reverse
        // unstable here means inplace and ordering of equal elements is not guaranteed
        result.sort_unstable_by(
            |a, b|
            a.partial_cmp(b)
                .unwrap()
                .reverse()
        );
        result
    }

    /// # Connects connected components (CCs)
    /// * returns vector of indices, where each corresponding node is in a different
    /// connected component
    pub(crate) fn suggest_connections(& self) -> Vec<usize>
    {
        let mut suggestions = Vec::new();
        let mut component_id : Vec<i32> = vec![-1; self.vertex_count()];
        let mut current_id = 0;
        for i in 0..self.vertex_count(){
            // already in a component?
            if component_id[i] != -1 {
                continue;
            }
            suggestions.push(i);

            // start depth first search over indices of vertices connected with vertex i
            for (j, _) in self.dfs_with_index(i) {
                component_id[j] = current_id;
            }
            current_id += 1;

        }
        suggestions
    }

    /// Count number of leaves in the graph, i.e. vertices with exactly one neighbor
    pub fn leaf_count(&self) -> usize {
        self.vertices
            .iter()
            .filter(|a| a.degree() == 1)
            .count()
    }

    /// * Creates String which contains the topology of the network in a format
    /// that can be used by **circo** etc. to generate a pdf of the graph.
    /// * **indices** are used as **labels**
    /// * search for **graphviz** to learn about **.dot** format
    #[deprecated(
        since = "0.3.0",
        note = "Please use any method of the `Dot` trait instead, e.g., `dot_with_indices`"
    )]
    pub fn to_dot(&self) -> String {
        let mut s = "graph{\n\t".to_string();

        for i in 0..self.vertex_count() {
            let _ = std::fmt::Write::write_fmt(&mut s, format_args!("{i} "));
        }
        s += "\n";

        for i in 0..self.vertex_count() {
            for &j in self.container(i).neighbors() {
                if i < j {
                    let _ = std::fmt::Write::write_fmt(&mut s, format_args!("\t{} -- {}\n", i, j));
                }
            }
        }
        s += "}";
        s
    }

    /// # Example
    /// ```
    /// use std::fs::File;
    /// use std::io::prelude::*;
    /// use net_ensembles::{Graph, EmptyNode, dot_constants::EXAMPLE_DOT_OPTIONS};
    ///
    /// let mut graph: Graph<EmptyNode> = Graph::new(3);
    /// graph.add_edge(0, 1).unwrap();
    /// graph.add_edge(0, 2).unwrap();
    /// graph.add_edge(1, 2).unwrap();
    ///
    /// // create string of dotfile
    /// let s = graph.to_dot_with_labels_from_contained(
    ///    EXAMPLE_DOT_OPTIONS,
    ///    |_contained, index| format!("Hey {}!", index)
    /// );
    ///
    /// // write to file
    /// let mut f = File::create("example.dot").expect("Unable to create file");
    /// f.write_all(s.as_bytes()).expect("Unable to write data");
    ///
    /// ```
    /// In this example, `example.dot` now contains:
    /// ```dot
    /// graph G{
    ///     bgcolor="transparent";
    ///     fontsize=50;
    ///     node [shape=ellipse, penwidth=1, fontname="Courier", pin=true ];
    ///     splines=true;
    ///     0 1 2 ;
    ///     "0" [label="Hey 0!"];
    ///     "1" [label="Hey 1!"];
    ///     "2" [label="Hey 2!"];
    ///     0 -- 1
    ///     0 -- 2
    ///     1 -- 2
    /// }
    /// ```
    ///
    /// Then you can use, e.g.,
    /// ```console
    /// foo@bar:~$ circo example.dot -Tpdf > example.pdf
    /// ```
    /// to create a **pdf** representation from it.
    /// Search for **graphviz** to learn more.
    #[deprecated(
        since = "0.3.0",
        note = "Please use any method of the `DotExtra` trait instead, e.g., `dot_from_contained_index`"
    )]
    pub fn to_dot_with_labels_from_contained<F, S1, S2>(&self, dot_options: S1, f: F ) -> String
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        F: Fn(&T, usize) -> S2
    {
        let mut writer = Vec::<u8>::with_capacity(40 * self.vertices.len());
        self.dot_from_contained_index(
            &mut writer,
            dot_options,
            |index, c|
            f(c, index)
        ).unwrap();
        String::from_utf8(writer)
            .unwrap()
    }

    /// # Same as `to_dot_with_labels_from_contained` but with access to neighbor information
    /// # Example
    /// ```
    /// use std::fs::File;
    /// use std::io::prelude::*;
    /// use net_ensembles::traits::*;
    /// use net_ensembles::dot_constants::*;
    /// use net_ensembles::{Graph,EmptyNode};
    ///
    /// let mut graph: Graph<EmptyNode> = Graph::new(5);
    /// graph.add_edge(0, 1).unwrap();
    /// graph.add_edge(0, 2).unwrap();
    /// graph.add_edge(1, 2).unwrap();
    /// graph.add_edge(0, 3).unwrap();
    /// graph.add_edge(3, 4).unwrap();
    ///
    /// // create string of dotfile
    /// let s = graph.to_dot_with_labels_from_container(
    ///     &[SPLINES, NO_OVERLAP].join("\n\t"),
    ///     |container, index|
    ///     {
    ///         container.contained();  // does nothing in this example, but you can still access
    ///                                 // contained, as you could in
    ///                                 // to_dot_with_labels_from_contained
    ///         format!("index {}, degree: {}", index, container.degree())
    ///     }
    /// );
    ///
    /// // write to file
    /// let mut f = File::create("example_2.dot").expect("Unable to create file");
    /// f.write_all(s.as_bytes()).expect("Unable to write data");
    ///
    /// ```
    /// In this example, `example_2.dot` now contains:
    /// ```dot
    /// graph G{
    ///     splines=true;
    ///     overlap=false;
    ///     0 1 2 3 4 ;
    ///     "0" [label="index 0, degree: 3"];
    ///     "1" [label="index 1, degree: 2"];
    ///     "2" [label="index 2, degree: 2"];
    ///     "3" [label="index 3, degree: 2"];
    ///     "4" [label="index 4, degree: 1"];
    ///     0 -- 1
    ///     0 -- 2
    ///     0 -- 3
    ///     1 -- 2
    ///     3 -- 4
    /// }
    /// ```
    ///
    /// Then you can use, e.g.,
    /// ```console
    /// foo@bar:~$ circo example_2.dot -Tpdf > example_2.pdf
    /// ```
    /// to create a **pdf** representation from it.
    /// Search for **graphviz** to learn more.
    #[deprecated(
        since = "0.3.0",
        note = "Please use any method of the `DotExtra` trait instead, e.g., `dot_from_container_index`"
    )]
    pub fn to_dot_with_labels_from_container<F, S1, S2>(&self, dot_options: S1, f: F ) -> String
        where
            S1: AsRef<str>,
            S2: AsRef<str>,
            F: Fn(&A, usize) -> S2,
    {
        let mut writer = Vec::<u8>::with_capacity(40 * self.vertices.len());
        self.dot_from_container_index(
            &mut writer,
            dot_options,
            |index, c|
            f(c, index)
        ).unwrap();
        String::from_utf8(writer)
            .unwrap()
    }

    /// * returns `None` **if** graph not connected **or** does not contain any vertices
    /// * uses repeated breadth first search
    pub fn diameter(&self) -> Option<usize> {
        if !self.is_connected()? {
            None
        } else {
            // well, then calculate from every node
            // (except 1 node) and use maximum found
            
            let mut max = 0;
            let mut bfs = self.bfs_index_depth(0);
            for index in 1..self.vertex_count() {
                let mut depth = 0;
                bfs.reuse(index);
                for (.., d) in &mut bfs {
                    depth = d;
                }
                max = max.max(depth);
            }

            Some(max)
        }
    }

    /// calculate the size of the longest shortest path **starting from** vertex with **index** `index`
    /// using breadth first search
    pub fn longest_shortest_path_from_index(&self, index: usize) -> Option<usize> {
        let (.., depth) = self.bfs_index_depth(index)
                            .last()?;
        Some(depth)
    }

    /// # calculate sizes of all binode connected components
    /// * returns (reverse) **ordered vector of sizes**
    /// i.e. the biggest component is of size `result[0]` and the smallest is of size `result[result.len() - 1]`
    /// * destroys the underlying topology and therefore moves `self`
    /// * if you still need your graph,
    /// use `self.clone().vertex_biconnected_components(false/true)` for your calculations
    /// # Definition: `vertex_biconnected_components(false)`
    /// Here, the (vertex) biconnected component of a graph is defined as maximal subset of nodes,
    /// where any one node could be removed and the remaining nodes would still be a connected component.
    /// ## Note
    /// Two vertices connected by an edge are considered to be biconnected, since after the
    /// removal of one vertex (and the corresponding edge), only one vertex remains.
    /// This vertex is in a connected component with itself.
    /// # Alternative Definition: `vertex_biconnected_components(true)`
    /// If you want to use the alternative definition:
    /// > The biconnected component is defined as maximal subset of vertices, where each vertex can be
    /// > reached by at least two node independent paths
    ///
    /// The alternative definition just removes all 2s from the result vector.
    /// # Citations
    /// I used the algorithm described in this paper:
    /// >  J. Hobcroft and R. Tarjan, "Algorithm 447: Efficient Algorithms for Graph Manipulation"
    /// > *Commun. ACM*, **16**:372-378, 1973, DOI: [10.1145/362248.362272](https://doi.org/10.1145/362248.362272)
    ///
    /// You can also take a look at:
    /// > M. E. J. Newman, "Networks: an Introduction" *Oxfort University Press*, 2010, ISBN: 978-0-19-920665-0.
    pub fn vertex_biconnected_components(mut self, alternative_definition: bool) -> Vec<usize> {

        let mut low: Vec<usize> = vec![0; self.vertex_count()];
        let mut number: Vec<usize> = vec![0; self.vertex_count()];
        let mut handled: Vec<bool> = vec![false; self.vertex_count()];
        let mut edge_stack: Vec<(usize, usize)> = Vec::with_capacity(self.vertex_count());
        let mut vertex_stack: Vec<usize> = Vec::with_capacity(self.vertex_count());
        let mut biconnected_components: Vec<Vec<(usize, usize)>> = Vec::new();

        let mut next_edge: (usize, usize);

        for pivot in 0..self.vertex_count() {

            if handled[pivot] {
                continue;
            }
            low[pivot] = 0;
            number[pivot] = 0;
            handled[pivot] = true;
            vertex_stack.push(pivot);

            while let Some(&top_vertex) = vertex_stack.last() {
                // if it has neighbors
                // does the vertex have neighbors?
                if self
                    .degree(top_vertex)
                    .unwrap() > 0
                    {
                        // remove one edge from graph, put it on stack
                        next_edge = (
                            top_vertex,
                            *self
                            .container(top_vertex)
                            .get_adj_first()
                            .unwrap()
                        );
                        edge_stack.push(next_edge);
                        let next_vertex = next_edge.1;
                        self.remove_edge(next_edge.0, next_edge.1).unwrap();

                        // check if next_vertex is not handled yet
                        if !handled[next_vertex] {
                            // number new point
                            number[next_vertex] = vertex_stack.len();
                            // add to stack of points
                            vertex_stack.push(next_edge.1);
                            // set LOWPOINT of the new point to NUMBER of current point
                            low[next_vertex] = number[top_vertex];
                            // now the point was visited once -> handled
                            handled[next_vertex] = true;
                        }
                        // Head of edge new point? NO -> Number of Head of edge lower than LOWPOINT of top point?
                        else if number[next_vertex] < low[top_vertex] {
                            // Set LOWPOINT of top Point to that number
                            low[top_vertex] = number[next_vertex];
                        }
                    }
                    // top point on stack has no edge
                    else {
                        vertex_stack.pop();
                        // at least one point in stack?
                        if let Some(&next_vertex) = vertex_stack.last() {
                            // LOWPOINT of top point equals NUMBER of next point on stack?
                            if low[top_vertex] == number[next_vertex]{
                                let mut tmp_component: Vec<(usize, usize)> = Vec::new();

                                while let Some(current_edge) = edge_stack.last() {
                                    if number[current_edge.1] < number[next_vertex]
                                    || number[current_edge.0] < number[next_vertex]
                                    {
                                        break;
                                    }
                                    tmp_component.push(*current_edge);
                                    edge_stack.pop();
                                }
                                // add to biconnected_components
                                if !tmp_component.is_empty(){
                                    biconnected_components.push(tmp_component);
                                }
                            }
                            // LOWPOINT of top point equals NUMBER of next point on stack? NO
                            else if low[top_vertex] < low[next_vertex] {
                                // Set LOWPOINT of next point equal LOWPOINT of current point if less
                                low[next_vertex] = low[top_vertex]
                            }

                        }
                        // no more vertices in stack stack?
                        else {
                            // exit loop
                            break;
                        }
                    }
                }
        }
        let mut result = Vec::with_capacity(biconnected_components.len());

        for component in biconnected_components {
            let mut size_set = HashSet::new();
            for edge in component {
                size_set.insert(edge.0);
                size_set.insert(edge.1);
            }
            result.push(size_set.len());
        }

        if alternative_definition {
            result.retain(|&val| val > 2);
        }
        // sort by reverse
        // unstable here means inplace and ordering of equal elements is not guaranteed
        result.sort_unstable_by(
            |a, b|
            a.partial_cmp(b)
                .unwrap()
                .reverse()
        );

        result
    }

    /// # Closely related (most of the time equal) to betweeness
    /// ## calculates vertex_load of all vertices in O(edges * vertices)
    /// * calculates the vertex_load for every vertex
    /// * defined as how many shortest paths pass through each vertex
    ///
    /// | variant             |                                                                                                                        |
    /// |---------------------|------------------------------------------------------------------------------------------------------------------------|
    /// | `vertex_load(true)`  | includes endpoints in calculation (for a complete graph with `N` vertices, every node will have vertex_load `N - 1`)  |
    /// | `vertex_load(false)` | excludes endpoints in calculation (for a complete graph with `N` vertices, every node will have vertex_load `0`)      |
    /// # Citations
    /// I used the algorithm described in
    /// > M. E. J. Newman, "Scientific collaboration networks. II. Shortest paths, weighted networks, and centrality",
    /// > Phys. Rev. E **64**, 016132, 2001, DOI: [10.1103/PhysRevE.64.016132](https://doi.org/10.1103/PhysRevE.64.016132)
    ///
    /// see also:
    /// > M. E. J. Newman, "Erratum: Scientific collaboration networks. II. Shortest paths, weighted networks, and centrality",
    /// > Phys. Rev. E **73**, 039906, 2006, DOI: [10.1103/PhysRevE.73.039906](https://doi.org/10.1103/PhysRevE.73.039906)
    pub fn vertex_load(&self, include_endpoints: bool) -> Vec<f64> {

        let mut queue0 = VecDeque::with_capacity(self.vertex_count());
        let mut queue1 = VecDeque::with_capacity(self.vertex_count());
        let mut ordering: Vec<usize> = Vec::with_capacity(self.vertex_count());
        let mut b = vec![0.0; self.vertex_count()];
        let mut b_k = vec![1f64; self.vertex_count()];
        let mut distance: Vec<Option<usize>> = vec![None; self.vertex_count()];
        let mut predecessor: Vec<Vec<usize>> = vec![Vec::new(); self.vertex_count()];
        

        for i in 0..self.vertex_count() {
            
            // initialize without allocation
            if i > 0 {
                for j in 0..self.vertex_count()
                {
                    b_k[j] = 1.0;
                    distance[j] = None;
                    // clear predecessors, way more efficient then new allocation
                    predecessor[j].clear();
                }
            }
            

            let mut depth = 0;
            queue0.push_back(i);
            distance[i] = Some(depth);


            // build up predecessor and ordering information
            while let Some(index) = queue0.pop_front() {
                ordering.push(index); // to get indices in reverse order of distance
                let container = self.container(index);
                for &neighbor in container.neighbors() {
                    if let Some(d) = distance[neighbor] {
                        if d == depth + 1 {
                            predecessor[neighbor].push(index);
                        }
                    }
                    // None
                    else {
                        distance[neighbor] = Some(depth + 1);
                        queue1.push_back(neighbor);
                        predecessor[neighbor].push(index);
                    }
                }
                if queue0.is_empty() {
                    std::mem::swap(&mut queue0, &mut queue1);
                    depth += 1;
                }
            }

            // calculate vertex_load resulting from the shortest paths starting at vertex i
            while let Some(index) = ordering.pop() {
                // skip last vertex
                if ordering.is_empty(){
                    break;
                }
                // add number of shortest path to total count

                b[index] += b_k[index];
                if !include_endpoints {
                    b[index] -= 1.0;
                }


                let fraction = b_k[index] / predecessor[index].len() as f64;
                for pred in predecessor[index].iter() {
                    b_k[*pred] += fraction;
                }
            }

        }
        b
    }

    pub fn closeness_centrality(&self) -> Vec<f64>
    {
        let mut count = vec![0; self.vertex_count()];

        let mut bfs = Bfs::new(self, 0);
        for i in 0..self.vertex_count()
        {
            bfs.reuse(i);
            for (index, _, depth) in &mut bfs{
                count[index] += depth;
            }
        }
        let val = (self.vertex_count() - 1) as f64;
        count.into_iter()
            .map(|count| val / count as f64)
            .collect()
    }

    /// # Calculates transitivity of graph
    /// * related to cluster coefficient (Note: transitivity and cluster coefficient are similar,
    /// but **not** necessarily equal)
    /// * returns `NaN`, if there are no paths of length two in the graph
    /// ## Definition
    /// > transitivity = (number of closed paths of length two) / (number of paths of length two)
    /// ## Citations
    /// For the definition see for example:
    /// > M. E. J. Newman, "Networks: an Introduction" *Oxfort University Press*, 2010, ISBN: 978-0-19-920665-0.
    pub fn transitivity(&self) -> f64 {
        let mut path_count: usize = 0;
        let mut closed_path_count: usize = 0;
        for source_index in 0..self.vertex_count() {
            for neighbor_1 in self
                                .container(source_index)
                                .neighbors()
            {
                for neighbor_2 in self
                                    .container(*neighbor_1)
                                    .neighbors()
                                    .filter(|&i| *i != source_index)  // do not use edge we came from
                {
                    if self
                        .container(*neighbor_2)
                        .is_adjacent(source_index)
                    {
                        closed_path_count += 1;
                    }
                    path_count += 1;
                }
            }
        }

        closed_path_count as f64 / path_count as f64
    }

    /// # Create a subgraph
    /// Method to create a subgraph. `node_list` should contain all the indices 
    /// corresponding to the nodes you want to keep, the order of the indices is 
    /// irrelevant. Duplicate indices will be ignored. 
    /// If any index is out of bounds (or `node_list` is empty), `None` will be returned
    /// 
    /// All edges between nodes that are inside the `node_list` will be kept.
    /// All other edges will be discarded. 
    /// The nodes in the subgraph will get new indices corresponding to their new position.
    /// The contained information (`T`) will be cloned.
    /// ## Note
    /// The container used will be changed to a [NodeContainer](crate::graph::NodeContainer),
    /// since I cannot guarantee that other container would make sense here. 
    /// E.g., if you used a [SwContainer](crate::sw_graph::SwContainer) the 
    /// information about the root edges might become invalid, because the corresponding 
    /// nodes might not be part of the subgraph
    pub fn cloned_subgraph(&self, mut node_list: Vec<usize>) -> Option<Graph<T>>
    where T: Clone
    {
        // get correct order
        node_list.sort_unstable();

        let last = *node_list.last()?;
        // check if biggest node is valid
        if last >= self.vertex_count() {
            return None;
        }

        // remove duplicates
        node_list.dedup();

        let map: HashMap<usize, usize> = node_list.iter()
            .copied()
            .zip(0..)
            .collect();

        let mut vertices = Vec::with_capacity(node_list.len());

        vertices.extend(
            node_list.into_iter()
                .enumerate()
                .map(
                    |(i, node_index)|
                    {
                        let container = self.container(node_index);
                        let mut adj = Vec::with_capacity(container.degree());
            
                        adj.extend(
                            container.neighbors()
                                .filter_map(|n| map.get(n))
                                .copied()
                            );
                        NodeContainer{
                            id: i,
                            adj,
                            node: container.contained().clone()
                        }
                    }
                )
        );

        let edge_count = vertices.iter()
            .map(NodeContainer::degree)
            .sum::<usize>() / 2;


        Some(
            Graph{
                next_id: vertices.len(),
                edge_count,
                vertices,
                phantom: PhantomData 
            }
        )
    }

}

impl<T, A> DotExtra<T, A> for GenericGraph<T, A>
where
    A: AdjContainer<T>,
{
    fn dot_from_container_index<F, S1, S2, W>(&self, mut writer: W, dot_options: S1, mut f: F)
        -> Result<(), std::io::Error>
        where
            S1: AsRef<str>,
            S2: AsRef<str>,
            F: FnMut(usize, &A) -> S2,
            W: Write
    {
        write!(writer, "graph G{{\n\t{}\n\t", dot_options.as_ref())?;

        for i in 0..self.vertex_count() {
            write!(writer, "{} ", i)?;
        }
        writeln!(writer, ";")?;

        for (index, container) in self.container_iter().enumerate() {
            let fun = f(index, container);
            writeln!(writer, "\t\"{}\" [label=\"{}\"];", index, fun.as_ref())?;
        }

        for i in 0..self.vertex_count() {
            for &j in self.container(i).neighbors() {
                if i < j {
                    writeln!(writer, "\t{} -- {}", i, j)?;

                }
            }
        }
        write!(writer, "}}")
    }

    fn dot_from_contained_index<F, S1, S2, W>(&self, writer: W, dot_options: S1, mut f: F)
        -> Result<(), std::io::Error>
        where
            W: Write,
            S1: AsRef<str>,
            S2: AsRef<str>,
            F: FnMut(usize, &T) -> S2
    {
        self.dot_from_container_index(
            writer,
            dot_options,
            |index, a| f(index, a.contained())
        )
    }
}

impl<T, A> Dot for GenericGraph<T, A>
where T: Node,
      A: AdjContainer<T>
{
    fn dot_from_indices<F, W, S1, S2>(&self, mut writer: W, dot_options: S1, mut f: F) -> Result<(), std::io::Error>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        W: Write,
        F: FnMut(usize) -> S2,
    {
        write!(writer, "graph G{{\n\t{}\n\t", dot_options.as_ref())?;

        for i in 0..self.vertex_count() {
            write!(writer, "{} ", i)?;
        }
        writeln!(writer, ";")?;

        for index in 0..self.vertex_count() {
            let fun = f(index);
            writeln!(writer, "\t\"{}\" [label=\"{}\"];", index, fun.as_ref())?;
        }

        for i in 0..self.vertex_count() {
            for &j in self.container(i).neighbors() {
                if i < j {
                    writeln!(writer, "\t{} -- {}", i, j)?;

                }
            }
        }
        write!(writer, "}}")
    }
}