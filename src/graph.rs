//! # Topology
//! Implements a network.
//!
//! You probably want to take a look at the struct `GenericGraph`,
//! since it contains the topology information.
//!
//! For Erdős-Rényi Graphs, see struct `ER`
use crate::{traits::*, GraphErrors, GenericGraph};
use std::marker::PhantomData;
use std::convert::From;

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

/// # Used for accessing neighbor information from graph
/// * contains Adjacency list
///  and internal id (normally the index in the graph).
/// * also contains user specified data, i.e, `T` from `NodeContainer<T>`
/// * see trait **`AdjContainer`**
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct NodeContainer<T: Node>{
    id: usize,
    pub(crate) adj: Vec<usize>,
    node: T,
}


impl<T: Node + SerdeStateConform> AdjContainer<T> for NodeContainer<T> {

    /// Create new instance with id
    fn new(id: usize, node: T) -> Self {
        NodeContainer{
            id,
            adj: Vec::new(),
            node,
        }
    }

    /// return reference to what the NodeContainer contains
    fn contained(&self) -> &T {
        &self.node
    }

    /// return mut reference to what the NodeContainer contains
    fn contained_mut(&mut self) -> &mut T {
        &mut self.node
    }

    /// returns iterator over indices of neighbors
    fn neighbors(&self) -> IterWrapper {
        IterWrapper::new_generic(self.adj.iter())
    }

    /// count number of neighbors, i.e. number of edges incident to `self`
    fn degree(&self) -> usize {
        self.adj.len()
    }

    /// returns id of container
    /// ## Note:
    /// (in `Graph<T>`: `id` equals the index corresponding to `self`)
    fn id(&self) -> usize {
        self.id
    }

    /// check if vertex with `other_id` is adjacent to self
    /// ## Note:
    /// (in `Graph<T>`: `id` equals the index corresponding to `self`)
    fn is_adjacent(&self, other_id: usize) -> bool {
        self.adj.contains(&other_id)
    }

    /// # Sorting adjecency lists
    /// * calls `sort_unstable()` on all adjecency lists
    fn sort_adj(&mut self) {
        self.adj.sort_unstable();
    }

    #[doc(hidden)]
    unsafe fn clear_edges(&mut self) {
        self.adj.clear();
    }

    #[doc(hidden)]
    unsafe fn push(&mut self, other: &mut Self)
        -> Result<(), GraphErrors>
    {
        if self.is_adjacent(other.id()) {
            return Err(GraphErrors::EdgeExists);
        }
        self.adj.push(other.id());
        other.adj.push(self.id);
        Ok(())
    }

    /// Tries to remove edges, returns error `GraphErrors::EdgeDoesNotExist` if impossible
    #[doc(hidden)]
    unsafe fn remove(&mut self, other: &mut Self)
        -> Result<(), GraphErrors>
    {
        if !self.is_adjacent(other.id()){
            return Err(GraphErrors::EdgeDoesNotExist);
        }

        self.swap_remove_element(other.id());
        other.swap_remove_element(self.id());

        Ok(())
    }

    fn get_adj_first(&self) -> Option<&usize> {
        self.adj.first()
    }
}

impl<T: Node> NodeContainer<T> {

    fn swap_remove_element(&mut self, elem: usize) {
        let index = self.adj
            .iter()
            .position(|&x| x == elem)
            .expect("swap_remove_element ERROR 0");

        self.adj
            .swap_remove(index);
    }
}

/// # Contains the topology and **implements functions** for analyzing topology
/// used for graph ensembles
/// # Example:
/// A graph, where each node stores a phase
/// ```
/// use net_ensembles::{Graph, Node, AdjContainer};
/// use net_ensembles::traits::DotExtra;
///
/// use std::fs::File;
///
/// // Note: feature "serde_support" is enabled on default
/// #[cfg(feature = "serde_support")]
/// use serde_json;
/// #[cfg(feature = "serde_support")]
/// use serde::{Serialize, Deserialize};
///
/// // define your own vertices, if you need to store extra information at each vertex
/// // if you do not use the feature "serde_support", you do not need to derive Serialize and Deserialize
/// #[derive(Debug, Clone)]
/// #[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
/// pub struct PhaseNode {phase: f64,}
///
/// // implement whatever you need
/// impl PhaseNode {
///     pub fn set_phase(&mut self, phase: f64) {
///         self.phase = phase;
///     }
///
///     pub fn get_phase(&self) -> f64 {
///         self.phase
///     }
/// }
///
/// // implement the trait `Node`
/// impl Node for PhaseNode {
///     fn new_from_index(index: usize) -> Self {
///         PhaseNode { phase: index as f64 * 10.0}
///     }
///
/// }
///
/// // now you can create an empty graph
/// let mut graph: Graph<PhaseNode> = Graph::new(4);
/// for i in 0..4 {
///     assert_eq!(
///       graph.at(i).get_phase(),
///       i as f64 * 10.0
///     );
/// }
///
/// // and fill it with edges
/// for i in 0..4 {
///     graph.add_edge(i, (i + 1) % 4).unwrap();
/// }
///
///
/// // you can manipulate the extra information stored at each Vertex
/// for i in 0..4 {
///     graph.at_mut(i).set_phase(i as f64 * 0.5);
/// }
///
/// // you can, of course, also access the information
/// for i in 0..4 {
///     assert_eq!(
///         graph.at(i).get_phase(),
///         i as f64 * 0.5
///     );
/// }
///
/// // if you want to visualize your graph, you can generate a file with graphviz representation
/// let mut f = File::create("phase_example.dot").expect("Unable to create file");
/// graph.dot_from_contained_index(
///     f,
///     "",
///     |index, contained|
///         format!(
///                 "Phase: {} at index {}",
///                 contained.get_phase(),
///                 index
///         )
/// ).unwrap();
///
/// // storing the graph only works, if the feature "serde_support" is enabled (enabled by default)
/// #[cfg(feature = "serde_support")]
/// {
///     let mut graph_file = File::create("store_graph_example.dat")
///            .expect("Unable to create file");
///     let s = serde_json::to_writer_pretty(graph_file, &graph).unwrap();
///
///     // loading stored graph:
///     let mut read_in = File::open("store_graph_example.dat")
///         .expect("Unable to open file");
///
///
///     let graph2:  Graph<PhaseNode> = serde_json::from_reader(read_in).unwrap();
///
///
///     // now, to show, that the graphs are equal, here is one of my test functions:
///     // modified for this example, which is a doc-test, so this example also serves as unit test
///
///     fn assert_equal_graphs(g1: &Graph<PhaseNode>, g2: &Graph<PhaseNode>) {
///         assert_eq!(g1.edge_count(), g2.edge_count());
///         assert_eq!(g1.vertex_count(), g2.vertex_count());
///         for (n0, n1) in g2.container_iter().zip(g1.container_iter()) {
///             assert_eq!(n1.id(), n0.id());
///             assert_eq!(n0.degree(), n1.degree());
///
///             for (i, j) in n1.neighbors().zip(n0.neighbors()) {
///                 assert_eq!(i, j);
///             }
///         }
///
///         for i in 0..g2.vertex_count() {
///             assert_eq!(
///                 g1.at(i).get_phase(),
///                 g2.at(i).get_phase()
///             );
///         }
///     }
///
///     // lets use it
///     assert_equal_graphs(&graph, &graph2);
///
///     // you can also clone the graph, if you need:
///     let clone = graph.clone();
///     assert_equal_graphs(&graph, &clone);
///     let clone2 = graph2.clone();
///     assert_equal_graphs(&clone, &clone2);
/// }
///
/// ```
/// `phase_example.dot` will contain
/// ```dot
/// graph G{
///
///     0 1 2 3 ;
///     "0" [label="Phase: 0 at index 0"];
///     "1" [label="Phase: 0.5 at index 1"];
///     "2" [label="Phase: 1 at index 2"];
///     "3" [label="Phase: 1.5 at index 3"];
///     0 -- 1
///     0 -- 3
///     1 -- 2
///     2 -- 3
/// }
/// ```
/// Now you can use `circo` or similar programs to create a pdf from that.
/// Search for **graphviz** for more info.
pub type Graph<T> = GenericGraph<T, NodeContainer<T>>;

impl<T> Graph<T>
where T: Node
{
    /// Efficiently create a complete graph with n nodes
    pub fn complete_graph(n: usize) -> Self
    {

        let mut vertices = Vec::with_capacity(n);
        for i in 0..n {
            let mut adj = Vec::with_capacity(n - 1);
            for index in 0..i {
                adj.push(index);
            }
            for index in (i + 1)..n {
                adj.push(index);
            }
            vertices.push(
                NodeContainer{
                    id: i,
                    node: T::new_from_index(i),
                    adj,
                }
            )
        }

        Self{
            next_id: n,
            edge_count: n*(n - 1) / 2,
            vertices,
            phantom: PhantomData::<T>,
        }
    }

    pub(crate) fn reset_from_graph(&mut self, other: &Self)
    {
        assert!(other.vertex_count() <= self.vertex_count());
        self.clear_edges();
        for i in 0..other.vertex_count(){
            self.vertices[i].adj.extend_from_slice(other.vertices[i].adj.as_slice());
        }
        self.edge_count = other.edge_count;

    }
}

impl<T: Node, A: AdjContainer<T>> From<&GenericGraph<T, A>> for Graph<T>
{
    fn from(source: &GenericGraph<T, A>) -> Self
    {
        // efficiently convert 
        let vertices = source
            .container_iter()
            .map(|container| 
                NodeContainer{
                    id: container.id(),
                    node: container.contained().clone(),
                    adj: container.neighbors().copied().collect(),
                }
            ).collect();
        Self{
            next_id: source.next_id,
            edge_count: source.edge_count(),
            vertices,
            phantom: PhantomData::<T>,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::EmptyNode;


    #[test]
    fn test_graph_container_push() {
        // create two nodes
        let mut c = NodeContainer::new(0, EmptyNode::new_from_index(0));
        let mut c2 = NodeContainer::new(1, EmptyNode::new_from_index(1));
        // create edge -> should not result in error!
        let res = unsafe { c.push(&mut c2) };
        if let Err(e) = res {
            panic!(format!("error: {}", e));
        }
        // now edge exists, should not be able to add it again:
        let res = unsafe { c.push(&mut c2) };
        assert!(res.is_err());

        assert_eq!(0, c.id());
    }

    #[test]
    fn correct_complete_graphs() {
        let g = Graph::<EmptyNode>::complete_graph(50);
        for index in 0..50 {
            let container = g.container(index);
            for neigbor in 0..50 {
                if neigbor == index {
                    assert!(!container.is_adjacent(neigbor));
                } else {
                    assert!(container.is_adjacent(neigbor));
                }
            }
        }
        assert_eq!(g.vertex_count(), 50);
    }


}
