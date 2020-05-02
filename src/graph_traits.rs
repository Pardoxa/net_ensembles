use std::fmt;
use std::borrow::Borrow;
use crate::IterWrapper;
use crate::sw::SwChangeState;
use crate::traits::SerdeStateConform;
use crate::GenericGraph;
/// What every node should be able to do
pub trait Node
where Self: Clone + SerdeStateConform {
    /// how to construct a blank object
    fn new_from_index(index: u32) -> Self;
}



/// Error messages
#[derive(Debug, Clone)]
pub enum GraphErrors{
    /// ### somehow, the existing of the edge is a problem
    /// Did you try to add an edge, which is already present?
    EdgeExists,
    /// ### ERROR 404: Edge not found ;)
    /// Did you try to delete a non existing edge?
    EdgeDoesNotExist,
}

impl GraphErrors {
    /// get error message as `&str`, for printing etc.
   pub fn to_str(&self) -> &'static str {
       match self {
           GraphErrors::EdgeExists          => &"EdgeExists",
           GraphErrors::EdgeDoesNotExist    => &"EdgeDoesNotExist",
       }
   }

   pub(crate) fn convert_to_sw_state(self) -> SwChangeState {
       SwChangeState::GError(self)
   }
}

impl fmt::Display for GraphErrors {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

/// Defines methods all adjecency containers should have
/// such that `GenericGraph` can use it
pub trait AdjContainer<T: Node>
{
    /// Create new instance with id
    fn new(id: u32, node: T) -> Self;


    /// return reference to what the AdjContainer contains
    fn contained(&self) -> & T;

    /// return mut reference to what the AdjContainer contains
    fn contained_mut(&mut self) -> &mut T;

    /// returns iterator over indices of neighbors
    fn neighbors(&self) -> IterWrapper;

    /// count number of neighbors, i.e. number of edges incident to `self`
    fn degree(&self) -> usize;

    /// returns id of container
    fn id(&self) -> u32;

    /// returns `Some(first element from the adjecency List)` or `None`
    fn get_adj_first(&self) -> Option<&u32>;

    /// check if vertex with `other_id` is adjacent to self
    /// ## Note:
    /// (in `Graph<T>`: `id` equals the index corresponding to `self`)
    fn is_adjacent(&self, other_id: u32) -> bool;

    /// Sorting adjecency lists
    fn sort_adj(&mut self);

    /// Remove all edges
    /// # Important
    /// * will not clear edges of other AdjContainer
    /// * only call this if you know exactly what you are doing
    #[doc(hidden)]
    unsafe fn clear_edges(&mut self);

    /// # What does it do?
    /// Creates edge in `self` and `other`s adjecency Lists
    /// # Why is it unsafe?
    /// * No logic to see, if AdjContainer are part of the same graph
    /// * Only intended for internal usage
    /// ## What should I do?
    /// * use members of `net_ensembles::GenericGraph` instead, that handles the logic
    #[doc(hidden)]
    unsafe fn push(&mut self, other: &mut Self)
        -> Result<(), GraphErrors>;

    /// # What does it do?
    /// Removes edge in `self` and `other`s adjecency Lists
    /// # Why is it unsafe?
    /// * No logic to see, if AdjContainer are part of the same graph
    /// * Only intended for internal usage
    /// ## What should I do?
    /// * use members of `net_ensembles::GenericGraph` instead, that handles the logic
    #[doc(hidden)]
    unsafe fn remove(&mut self, other: &mut Self)
        -> Result<(), GraphErrors>;
}


/// Trait for measuring topological properties of a Graph
pub trait MeasurableGraphQuantities<T, A>
where
    T: Node,
    A: AdjContainer<T>
{
    /// calculates the average degree of the graph
    /// * `(2 * edge_count) / vertex_count`
    fn average_degree(&self) -> f32;

    /// returns number of vertices adjacent to vertex `index`
    fn degree(&self, index: usize) -> Option<usize>;

    /// # compute sizes of all *connected components*
    ///
    /// * the **number** of connected components is the **size** of the returned vector, i.e. `result.len()`
    /// * returns **empty** vector, if graph does not contain vertices
    /// * returns (reverse) **ordered vector of sizes** of the connected components,
    /// i.e. the biggest component is of size `result[0]` and the smallest is of size `result[result.len() - 1]`
    fn connected_components(&self) -> Vec<u32>;

    /// * returns `None` **if** graph not connected **or** does not contain any vertices
    /// * uses repeated breadth first search
    fn diameter(&self) -> Option<u32>;

    /// returns total number of edges in graph
    fn edge_count(&self) -> u32;

    /// | result       |                          condition                       |
    /// |--------------|----------------------------------------------------------|
    /// | `None`       | **if** graph does not contain any vertices               |
    /// | `Some(true)` | **else if** all vertices are connected by paths of edges |
    /// | `Some(false)`| **otherwise**                                            |
    fn is_connected(&self) -> Option<bool>;

    /// Count number of leaves in the graph, i.e. vertices with exactly one neighbor
    fn leaf_count(&self) -> usize;

    /// calculate the size of the longest shortest path **starting from** vertex with **index** `index`
    /// using breadth first search
    fn longest_shortest_path_from_index(&self, index: u32) -> Option<u32>;

    /// # definition
    /// Calculates the size of the **q-core** (i.e. number of nodes in the biggest possible set of nodes,
    /// where all nodes from the set are connected with at least `q` other nodes from the set)
    ///
    /// returns `None` if impossible to calculate (e.g. `vertex_count == 0` or `q <= 1`)
    fn q_core(&self, q: u32) -> Option<u32>;

    /// # Calculates transitivity of graph
    /// * related to cluster coefficient (Note: transitivity and cluster coefficient are similar,
    /// but **not** necessarily equal)
    /// * returns `NaN`, if there are no paths of length two in the graph
    /// ## Definition
    /// > transitivity = (number of closed paths of length two) / (number of paths of length two)
    /// ## Citations
    /// For the definition see for example:
    /// > M. E. J. Newman, "Networks: an Introduction" *Oxfort University Press*, 2010, ISBN: 978-0-19-920665-0.
    fn transitivity(&self) -> f64;

    /// returns number of vertices present in graph
    fn vertex_count(&self) -> u32;


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
    fn vertex_load(&self, include_endpoints: bool) -> Vec<f64>;

}


impl<T, A, E> MeasurableGraphQuantities<T, A> for E
where
    E: Borrow<GenericGraph<T, A>>,
    T: Node,
    A: AdjContainer<T>
{
    fn average_degree(&self) -> f32 {
        self.borrow().average_degree()
    }

    fn degree(&self, index: usize) -> Option<usize> {
        self.borrow().degree(index)
    }

    fn connected_components(&self) -> Vec<u32> {
        self.borrow().connected_components()
    }

    fn diameter(&self) -> Option<u32> {
        self.borrow().diameter()
    }

    fn edge_count(&self) -> u32 {
        self.borrow().edge_count()
    }

    fn is_connected(&self) -> Option<bool> {
        self.borrow().is_connected()
    }

    fn leaf_count(&self) -> usize {
        self.borrow().leaf_count()
    }

    fn longest_shortest_path_from_index(&self, index: u32) -> Option<u32> {
        self.borrow().longest_shortest_path_from_index(index)
    }

    fn q_core(&self, q: u32) -> Option<u32>{
        self.borrow().q_core(q)
    }

    fn transitivity(&self) -> f64 {
        self.borrow().transitivity()
    }

    fn vertex_count(&self) -> u32 {
        self.borrow().vertex_count()
    }

    fn vertex_load(&self, include_endpoints: bool) -> Vec<f64> {
        self.borrow().vertex_load(include_endpoints)
    }
}
