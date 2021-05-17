use std::fmt;
use crate::IterWrapper;
use crate::sw::SwChangeState;
use crate::traits::SerdeStateConform;
use crate::GenericGraph;

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

/// What every node should be able to do
pub trait Node
where Self: Clone + SerdeStateConform {
    /// how to construct a blank object
    fn new_from_index(index: usize) -> Self;
}



/// Error messages
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
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
pub trait AdjContainer<T>
{
    /// Create new instance with id
    fn new(id: usize, node: T) -> Self;


    /// return reference to what the AdjContainer contains
    fn contained(&self) -> & T;

    /// return mut reference to what the AdjContainer contains
    fn contained_mut(&mut self) -> &mut T;

    /// returns iterator over indices of neighbors
    fn neighbors(&self) -> IterWrapper;

    /// count number of neighbors, i.e. number of edges incident to `self`
    fn degree(&self) -> usize;

    /// returns id of container
    fn id(&self) -> usize;

    /// returns `Some(first element from the adjecency List)` or `None`
    fn get_adj_first(&self) -> Option<&usize>;

    /// check if vertex with `other_id` is adjacent to self
    /// ## Note:
    /// (in `Graph<T>`: `id` equals the index corresponding to `self`)
    fn is_adjacent(&self, other_id: usize) -> bool;

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

/// Get the adjacency list of a AdjContainer
pub trait AdjList<Edge>
{
    fn edges(&self) -> &[Edge];
}


/// Trait for measuring topological properties of a Graph
pub trait MeasurableGraphQuantities<G>
{
    /// calculates the average degree of the graph
    /// * `(2 * edge_count) / vertex_count`
    fn average_degree(&self) -> f32;

    /// * returns number of vertices adjacent to vertex `index`
    /// * `None` if index out of bounds
    fn degree(&self, index: usize) -> Option<usize>;

    /// # compute sizes of all *connected components*
    ///
    /// * the **number** of connected components is the **size** of the returned vector, i.e. `result.len()`
    /// * returns **empty** vector, if graph does not contain vertices
    /// * returns (reverse) **ordered vector of sizes** of the connected components,
    /// i.e. the biggest component is of size `result[0]` and the smallest is of size `result[result.len() - 1]`
    fn connected_components(&self) -> Vec<usize>;

    /// * returns `None` **if** graph not connected **or** does not contain any vertices
    /// * uses repeated breadth first search
    fn diameter(&self) -> Option<usize>;

    /// returns total number of edges in graph
    fn edge_count(&self) -> usize;

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
    fn longest_shortest_path_from_index(&self, index: usize) -> Option<usize>;

    /// # definition
    /// Calculates the size of the **q-core** (i.e. number of nodes in the biggest possible set of nodes,
    /// where all nodes from the set are connected with at least `q` other nodes from the set)
    ///
    /// returns `None` if impossible to calculate (e.g. `vertex_count == 0` or `q <= 1`)
    fn q_core(&self, q: usize) -> Option<usize>;

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
    fn vertex_biconnected_components(&self, alternative_definition: bool) -> Vec<usize>;

    /// returns number of vertices present in graph
    fn vertex_count(&self) -> usize;


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


impl<T, A, E> MeasurableGraphQuantities<GenericGraph<T, A>> for E
where
    T: Node,
    A: AdjContainer<T>,
    GenericGraph<T, A>: Clone,
    E: AsRef<GenericGraph<T, A>>,
{
    fn average_degree(&self) -> f32 {
        self.as_ref().average_degree()
    }

    fn degree(&self, index: usize) -> Option<usize> {
        self.as_ref().degree(index)
    }

    fn connected_components(&self) -> Vec<usize> {
        self.as_ref().connected_components()
    }

    fn diameter(&self) -> Option<usize> {
        self.as_ref().diameter()
    }

    fn edge_count(&self) -> usize {
        self.as_ref().edge_count()
    }

    fn is_connected(&self) -> Option<bool> {
        self.as_ref().is_connected()
    }

    fn leaf_count(&self) -> usize {
        self.as_ref().leaf_count()
    }

    fn longest_shortest_path_from_index(&self, index: usize) -> Option<usize> {
        self.as_ref().longest_shortest_path_from_index(index)
    }

    fn q_core(&self, q: usize) -> Option<usize>{
        self.as_ref().q_core(q)
    }

    fn transitivity(&self) -> f64 {
        self.as_ref().transitivity()
    }

    fn vertex_biconnected_components(&self, alternative_definition: bool) -> Vec<usize> {
        let clone = (*self.as_ref()).clone();
        clone.vertex_biconnected_components(alternative_definition)
    }

    fn vertex_count(&self) -> usize {
        self.as_ref().vertex_count()
    }

    fn vertex_load(&self, include_endpoints: bool) -> Vec<f64> {
        self.as_ref().vertex_load(include_endpoints)
    }
}
