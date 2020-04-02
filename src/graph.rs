//! # Topology
//! Implements a network.
//!
//! You probably want to take a look at the struct `GenericGraph`,
//! since it contains the topology information.
//!
//! For Erdős-Rényi Graphs, see struct `ER`
use std::fmt;
use crate::traits::*;
use crate::GraphErrors;
use crate::GenericGraph;


/// # Used for accessing neighbor information from graph
/// * contains Adjacency list
///  and internal id (normally the index in the graph).
/// * also contains user specified data, i.e, `T` from `NodeContainer<T>`
/// * see trait **`AdjContainer`**
#[derive(Debug, Clone)]
pub struct NodeContainer<T: Node>{
    id: u32,
    adj: Vec<u32>,
    node: T,
}

const PARSE_ID: &str        = "id: ";
const PARSE_SEPERATOR: &str = "\t";
const PARSE_ADJ: &str       = "adj: ";
const PARSE_NODE: &str      = "Node: ";

impl<T: Node> fmt::Display for NodeContainer<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // "id: {} adj: {:?} Node: {}"
        write!(
            f,
            "{}{}{}{}{:?}{}{}{}" ,
            PARSE_ID, self.id, PARSE_SEPERATOR,
            PARSE_ADJ, self.adj, PARSE_SEPERATOR,
            PARSE_NODE, self.node.make_string()
                .expect(&format!("make_string failed - Did you forget to Override? Look at {}::Node", env!("CARGO_PKG_NAME")))
        )
    }
}





impl<T: Node> AdjContainer<T> for NodeContainer<T> {

    /// Create new instance with id
    fn new(id: u32, node: T) -> Self {
        NodeContainer{
            id,
            adj: Vec::new(),
            node,
        }
    }

    /// # parse from str
    /// * tries to parse a NodeContainer from a `str`.
    /// * will ignore leading whitespaces and other chars, as long as they do not match `"id: "`
    /// * returns `None` if failed
    ///
    /// ## Return
    /// 1) returns string slice beginning directly after the part, that was used to parse
    /// 2) the `NodeContainer` resulting form the parsing
    fn parse_str(to_parse: &str) -> Option<(&str, Self)> {
        // skip identifier PARSE_ID
        let mut split_index = to_parse.find(PARSE_ID)?;
        split_index += PARSE_ID.len();
        let remaining_to_parse = &to_parse[split_index..];

        // find index of next PARSE_SEPERATOR and split there
        split_index = remaining_to_parse.find(PARSE_SEPERATOR)?;
        let (id_str, mut remaining_to_parse) = remaining_to_parse.split_at(split_index);
        let id = id_str.parse::<u32>().ok()?;

        // skip to after nex identifier
        split_index = remaining_to_parse.find(PARSE_ADJ)?;
        split_index += PARSE_ADJ.len();
        remaining_to_parse = &remaining_to_parse[split_index..];


        // find index of next PARSE_SEPERATOR and split there
        split_index = remaining_to_parse.find(PARSE_SEPERATOR)?;
        let (mut adj_str, mut remaining_to_parse) = remaining_to_parse.split_at(split_index);
        // now remove the brackets []
        adj_str = &adj_str[1..adj_str.len() - 1];
        // now split at ", "
        // collect into vector, which is None, if something failed
        let adj_option: Option<Vec<u32>> = adj_str
                                            .split(", ")
                                            .map(|x| x.parse::<u32>().ok())
                                            .collect();
        let adj = adj_option?;

        // skip until printed node
        split_index = remaining_to_parse.find(PARSE_NODE)?;
        split_index += PARSE_NODE.len();
        remaining_to_parse = &remaining_to_parse[split_index..];

        if let Some((remaining_to_parse, node)) = Node::parse_str(remaining_to_parse) {
            let result = NodeContainer{
                id,
                adj,
                node,
            };
            Some((remaining_to_parse, result))
        } else {
            None
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
    fn id(&self) -> u32 {
        self.id
    }

    /// check if vertex with `other_id` is adjacent to self
    /// ## Note:
    /// (in `Graph<T>`: `id` equals the index corresponding to `self`)
    fn is_adjacent(&self, other_id: &u32) -> bool {
        self.adj.contains(other_id)
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
        if self.is_adjacent(&other.id()) {
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
        if !self.is_adjacent(&other.id()){
            return Err(GraphErrors::EdgeDoesNotExist);
        }

        self.swap_remove_element(other.id());
        other.swap_remove_element(self.id());

        Ok(())
    }

    fn get_adj_first(&self) -> Option<&u32> {
        self.adj.first()
    }
}

impl<T: Node> NodeContainer<T> {

    fn swap_remove_element(&mut self, elem: u32) -> () {
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
/// use net_ensembles::{traits::*,Graph};
/// use std::fs::File;
/// use std::io::prelude::*;
/// // define your own vertices, if you need to store extra information at each vertex
/// #[derive(Debug, Clone)]
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
///     fn new_from_index(index: u32) -> Self {
///         PhaseNode { phase: index as f64 * 10.0}
///     }
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
/// // if you want to visualize your graph, you can generate a string with graphviz representation
/// let dot = graph.to_dot_with_labels_from_contained(
///     "",
///     |contained, index|
///            format!(
///                 "Phase: {} at index {}",
///                 contained.get_phase(),
///                 index
///             )
/// );
/// // which you can store in a dot file
/// let mut f = File::create("phase_example.dot").expect("Unable to create file");
/// f.write_all(dot.as_bytes()).expect("Unable to write data");
///
/// // or just print it out
/// println!("{}", dot);
/// ```
/// `phase_example.dot` will then contain
/// ```dot
/// graph G{
///
/// 	0 1 2 3 ;
/// 	"0" [label="Phase: 0 at index 0"];
/// 	"1" [label="Phase: 0.5 at index 1"];
/// 	"2" [label="Phase: 1 at index 2"];
/// 	"3" [label="Phase: 1.5 at index 3"];
/// 	0 -- 1
/// 	0 -- 3
/// 	1 -- 2
/// 	2 -- 3
/// }
/// ```
/// Now you can use `circo` or similar programs to create a pdf from that.
/// Search for **graphviz** for more info.
/// # Example 2
/// * if you also want to be able to store your graph, you have to override two functions of the Node trait, as shown below
/// ```
/// use net_ensembles::{Node,Graph,traits::*};
/// use std::fs::File;
/// use std::io::prelude::*;
/// // define your own vertices, if you need to store extra information at each vertex
/// #[derive(Debug, Clone)]
/// pub struct PhaseNode {phase: f64,}
///
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
/// impl Node for PhaseNode {
///     fn new_from_index(index: u32) -> Self {
///         PhaseNode { phase: 10.0 * index as f64 }
///     }
///
///     // Override this, to save the graph
///     fn make_string(&self) -> Option<String> {
///         Some(format!("phase: {},", self.phase))
///     }
///
///     // Override this, since you want to load the stored graph
///     fn parse_str(to_parse: &str) -> Option<(&str, Self)>
///         where Self: Sized
///     {
///         let identifier = "phase: ";
///         // searching for identifier
///         let mut split_index = to_parse.find(identifier)?;
///
///         // add length of identifier to index
///         split_index += identifier.len();
///
///         // new string slice to skip to our identifier
///         let remaining_to_parse = &to_parse[split_index..];
///
///         // find out, where our data ends
///         split_index = remaining_to_parse.find(",")?;
///
///         // create new string slice, beginning after what Node::make_string() created
///         let (phase_str, mut remaining_to_parse) = remaining_to_parse.split_at(split_index);
///         remaining_to_parse = &remaining_to_parse[1..];
///
///         // parse our data
///         let phase = phase_str.parse::<f64>().ok()?;
///         let node = PhaseNode{ phase };
///
///         // return our struct as option
///         Some((remaining_to_parse, node))
///     }
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
/// // if you want to visualize your graph, you can generate a string with graphviz representation
/// let dot = graph.to_dot_with_labels_from_contained(
///     "",
///     |contained, index|
///            format!(
///                 "Phase: {} at index {}",
///                 contained.get_phase(),
///                 index
///             )
/// );
/// // which you can store in a dot file
/// let mut f = File::create("phase_example_still_works.dot").expect("Unable to create file");
/// f.write_all(dot.as_bytes()).expect("Unable to write data");
///
/// // or just print it out
/// println!("{}", dot);
///
/// // if you want to store your graph, you can do the following:
/// // first get string representation
/// let s = graph.to_string();
/// // open file
/// let mut graph_file = File::create("store_graph_example.dat").expect("Unable to create file");
/// // write to file
/// graph_file.write_all(s.as_bytes()).expect("Unable to write data");
/// // close file
/// drop(graph_file);
///
/// // now, if you want to load your network:
/// let mut read_in = File::open("store_graph_example.dat").expect("unable to open file");
/// let mut test_data = String::new();
/// read_in.read_to_string(&mut test_data).expect("unable to read file");
///
/// // now we read the string. We still have to parse it:
/// let (_, graph2) = Graph::<PhaseNode>::parse_str(&test_data).unwrap();
///
/// // now, to show, that the graphs are equal, here is one of my test functions:
/// // modified for this example, which is a doc-test, so this example also serves as unit test
///
/// fn assert_equal_graphs(g1: &Graph<PhaseNode>, g2: &Graph<PhaseNode>) {
///     assert_eq!(g1.edge_count(), g2.edge_count());
///     assert_eq!(g1.vertex_count(), g2.vertex_count());
///     for (n0, n1) in g2.container_iter().zip(g1.container_iter()) {
///         assert_eq!(n1.id(), n0.id());
///         assert_eq!(n0.degree(), n1.degree());
///
///         for (i, j) in n1.neighbors().zip(n0.neighbors()) {
///             assert_eq!(i, j);
///         }
///     }
///
///     for i in 0..g2.vertex_count() as usize {
///         assert_eq!(
///             g1.at(i).get_phase(),
///             g2.at(i).get_phase()
///         );
///     }
/// }
/// // lets use it
/// assert_equal_graphs(&graph, &graph2);
///
/// // you can also clone the graph, if you need:
/// let clone = graph.clone();
/// assert_equal_graphs(&graph, &clone);
/// let clone2 = graph2.clone();
/// assert_equal_graphs(&clone, &clone2);
/// ```
pub type Graph<T> = GenericGraph<T, NodeContainer<T>>;


#[cfg(test)]
mod tests {
    use super::*;
    use crate::TestNode;


    #[test]
    fn test_graph_container_push() {
        // create two nodes
        let mut c = NodeContainer::new(0, TestNode::new_from_index(0));
        let mut c2 = NodeContainer::new(1, TestNode::new_from_index(1));
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
    fn parsing_invalid_node_container() {
        // parsing gibberish should return None, not panic!
        let s = "geufgeiruwgeuwhuiwehfoipaerughpsiucvhuirhgvuir";
        let res = NodeContainer::<TestNode>::parse_str(&s);
        assert!(res.is_none());
    }

    #[test]
    #[should_panic]
    fn test_printing_default() {
        let mut graph: Graph<TestNode> = Graph::new(20);
        graph.add_edge(0, 1).unwrap();

        println!("{}", graph.container(0));
    }

}
