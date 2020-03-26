//! # Topology
//! Implements a network.
//!
//! You probably want to take a look at the struct `Graph`
use std::fmt;
use crate::node::Node;
use std::cmp::max;
use std::convert::TryFrom;
use std::collections::VecDeque;

/// # constant for dot options
/// ```
/// pub const DEFAULT_DOT_OPTIONS: &str = "bgcolor=\"transparent\";\n\tfontsize=50;\n\t\
///         node [shape=ellipse, penwidth=1, \
///         fontname=\"Courier\", pin=true ];\n\tsplines=true;";
/// ```
pub const DEFAULT_DOT_OPTIONS: &str = "bgcolor=\"transparent\";\n\tfontsize=50;\n\t\
        node [shape=ellipse, penwidth=1, \
        fontname=\"Courier\", pin=true ];\n\tsplines=true;";

/// Error messages
#[derive(Debug)]
pub enum GraphErrors{
    /// ### somehow, the existing of the edge is a problem
    /// Did you try to add an edge, which is already present, to the graph
    EdgeExists,
    /// ### ERROR 404: Edge not found ;)
    /// Did you try to delete a non existing edge?
    EdgeDoesNotExist,
    /// ### Have you tried a smaller index?
    IndexOutOfRange,
    /// ### No self loops allowed!
    /// Meaning you can't: `graph.add_edge(i, i);`
    IdenticalIndices,
}

impl GraphErrors {
    /// get error message as `&str`, for printing etc.
   pub fn to_str(&self) -> &'static str {
       match self {
           GraphErrors::EdgeExists          => &"EdgeExists",
           GraphErrors::EdgeDoesNotExist    => &"EdgeDoesNotExist",
           GraphErrors::IndexOutOfRange     => &"IndexOutOfRange",
           GraphErrors::IdenticalIndices    => &"IdenticalIndices",
       }
   }
}

impl fmt::Display for GraphErrors {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

/// # Used for accessing neighbor information from graph
/// Contains Adjacency list
///  and internal id. Normally the index in the graph.
///
///
/// Also contains user specified data, i.e, `T` from `NodeContainer<T>`
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


impl<T: Node> NodeContainer<T> {
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
    pub fn parse_str(to_parse: &str) -> Option<(&str, Self)> {
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
    pub fn get_contained(&self) -> &T {
        &self.node
    }

    /// return mut reference to what the NodeContainer contains
    pub fn get_contained_mut(&mut self) -> &mut T {
        &mut self.node
    }

    /// returns iterator over indices of neighbors
    pub fn neighbors(&self) -> std::slice::Iter::<u32> {
        self.adj.iter()
    }

    /// count number of neighbors, i.e. number of edges incident to `self`
    pub fn neighbor_count(&self) -> usize {
        self.adj.len()
    }

    /// returns id of container
    /// ## Note:
    /// (in `Graph<T>`: `id` equals the index corresponding to `self`)
    pub fn get_id(&self) -> u32 {
        self.id
    }

    /// check if vertex with `other_id` is adjacent to self
    /// ## Note:
    /// (in `Graph<T>`: `id` equals the index corresponding to `self`)
    pub fn is_adjacent(&self, other_id: &u32) -> bool {
        self.adj.contains(other_id)
    }

    fn push(&mut self, other: &mut NodeContainer<T>) -> Result<(),GraphErrors> {
        if self.is_adjacent(&other.get_id()) {
            return Err(GraphErrors::EdgeExists);
        }
        self.adj.push(other.get_id());
        other.adj.push(self.id);
        Ok(())
    }

    fn swap_delete_element(&mut self, elem: u32) -> () {
        let index = self.adj
            .iter()
            .position(|x| *x == elem)
            .expect("swap_delete_element ERROR 0");

        self.adj
            .swap_remove(index);
    }

    /// Tries to remove edges, returns error `GraphErrors::EdgeDoesNotExist` if impossible
    fn remove(&mut self, other: &mut NodeContainer<T>) -> Result<(),GraphErrors> {
        if !self.is_adjacent(&other.get_id()){
            return Err(GraphErrors::EdgeDoesNotExist);
        }

        self.swap_delete_element(other.get_id());
        other.swap_delete_element(self.get_id());

        Ok(())
    }
}

/// # Contains the topology and **implements functions** for analyzing topology
/// used for graph ensembles
/// # Example:
/// A graph, where each node stores a phase
/// ```
/// use net_ensembles::{Node,Graph};
/// use std::fs::File;
/// use std::io::prelude::*;
/// // define your own vertices, if you need to store extra information at each vertex
/// #[derive(Debug)]
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
/// let dot = graph.to_dot_with_labels(
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
/// * if you want to be able to clone the graph, only `#[derive(Clone)]`
/// * if you also want to be able to store your graph, you have to override two functions of the Node trait, as shown below
/// ```
/// use net_ensembles::{Node,Graph};
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
/// let dot = graph.to_dot_with_labels(
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
///     for (n0, n1) in g2.node_container_iter().zip(g1.node_container_iter()) {
///         assert_eq!(n1.get_id(), n1.get_id());
///         assert_eq!(n1.neighbor_count(), n1.neighbor_count());
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
#[derive(Debug, Clone)]
pub struct Graph<T: Node> {
    next_id: u32,
    edge_count: u32,
    vertices: Vec<NodeContainer<T>>,
}

impl<T: Node> fmt::Display for Graph<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        for v in self.node_container_iter() {
            s += &format!("{}\n", v);
        }
        write!(f, "next_id: {}\nedge_count: {}\nvertices:\n{}", self.next_id, self.edge_count, s)
    }
}

impl<T: Node> Graph<T> {
    /// Create new graph with `size` nodes
    /// and no edges
    pub fn new(size: u32) -> Self {
        let mut vertices = Vec::with_capacity(size as usize);
        for i in 0..size {
            let container = NodeContainer::new(i, T::new_from_index(i));
            vertices.push(container);
        }
        Self{
            vertices,
            next_id: size,
            edge_count: 0,
        }
    }

    /// # parse from str
    /// * tries to parse `Graph` from a `str`.
    /// * will ignore leading whitespaces and other chars, as long as they do not match `"next_id: "`
    /// * returns `None` if failed
    ///
    /// ## Return
    /// 1) returns string slice beginning directly after the part, that was used to parse
    /// 2) the `Graph` resulting form the parsing
    pub fn parse_str(to_parse: &str) -> Option<(&str, Self)> {
        // skip identifier
        let mut split_index = to_parse.find("next_id: ")? + 9;
        let remaining_to_parse = &to_parse[split_index..];

        // find index of next PARSE_SEPERATOR and split there
        split_index = remaining_to_parse.find("\n")?;
        let (next_id_str, mut remaining_to_parse) = remaining_to_parse.split_at(split_index);
        let next_id = next_id_str.parse::<u32>().ok()?;

        // skip identifier
        split_index = remaining_to_parse.find("edge_count: ")? + 12;
        remaining_to_parse = &remaining_to_parse[split_index..];

        // find index of next PARSE_SEPERATOR and split there
        split_index = remaining_to_parse.find("\n")?;
        let (edge_count_str, mut remaining_to_parse) = remaining_to_parse.split_at(split_index);
        let edge_count = edge_count_str.parse::<u32>().ok()?;

        // Parse the vertex vector
        let mut vertices = Vec::with_capacity(next_id as usize);
        for _ in 0..next_id {
            let result = NodeContainer::<T>::parse_str(&remaining_to_parse)?;
            remaining_to_parse = result.0;
            let node = result.1;

            vertices.push(node);
        }
        Some((
                remaining_to_parse,
                Self{
                    vertices,
                    next_id,
                    edge_count,
                }
            ))
    }

    /// borrows NodeContainer at index
    pub fn get_node_container(&self, index: usize) -> &NodeContainer<T> {
        &self.vertices[index]
    }

    /// get iterator over NodeContainer in order of the indices
    pub fn node_container_iter(&self) -> std::slice::Iter::<NodeContainer<T>> {
        self.vertices.iter()
    }


    fn get_node_container_mut(&mut self, index: usize) -> &mut NodeContainer<T> {
        &mut self.vertices[index]
    }

    /// # For your calculations etc.
    /// * **read access** to **your struct** T, stored at **each vertex**, that implements `Node` trait
    /// * see first **code example** (beginning of this page)
    pub fn at(&self, index: usize) -> &T {
        self.get_node_container(index).get_contained()
    }

    /// # For your calculations etc.
    /// * **write access** to **your struct** T, stored at **each vertex**, that implements `Node` trait
    /// * see first **code example** (beginning of this page)
    pub fn at_mut(&mut self, index: usize) -> &mut T {
        self.get_node_container_mut(index).get_contained_mut()
    }

    /// returns number of vertices present in graph
    pub fn vertex_count(&self) -> u32 {
        self.next_id
    }

    pub fn average_neighbor_count(&self) -> f32 {
        (2 * self.edge_count()) as f32 / self.vertex_count() as f32
    }

    /// Returns two mutable references in a tuple
    /// ## ErrorCases:
    /// `GraphErrors::IndexOutOfRange`  <-- index to large
    /// GraphErrors::IdenticalIndices <-- index1 == index2 not allowed!
    fn get_2_mut(&mut self, index1: u32, index2: u32) ->
        Result<(&mut NodeContainer<T>, &mut NodeContainer<T>),GraphErrors>
    {
        if index1 >= self.next_id || index2 >= self.next_id {
            return Err(GraphErrors::IndexOutOfRange);
        } else if index1 == index2 {
            return Err(GraphErrors::IdenticalIndices);
        }
        let r1: &mut NodeContainer<T>;
        let r2: &mut NodeContainer<T>;

        let ptr = self.vertices.as_mut_ptr();

        unsafe {
            r1 = &mut *ptr.offset(index1 as isize);
            r2 = &mut *ptr.offset(index2 as isize);
        }

        Ok((r1, r2))
    }

    /// Adds edge between nodes `index1` and `index2`
    /// ## ErrorCases:
    /// | Error | Reason |
    /// | ---- | ---- |
    /// | `GraphErrors::IndexOutOfRange` | `index1` or `index2` larger than `self.vertex_count()`  |
    /// | `GraphErrors::IdenticalIndices` | `index2 == index1` not allowed! |
    /// | `GraphErrors::EdgeExists` | requested edge already exists! |
    pub fn add_edge(&mut self, index1: u32, index2: u32) -> Result<(),GraphErrors> {
        let (r1, r2) = self.get_2_mut(index1, index2)?;
        r1.push(r2)?;
        self.edge_count += 1;
        Ok(())
    }

    /// Removes edge between nodes *index1* and *index2*
    /// ## ErrorCases:
    /// | Error | Reason |
    /// | ---- | ---- |
    /// | `GraphErrors::IndexOutOfRange` | `index1` or `index2` larger than `self.vertex_count()`  |
    /// | `GraphErrors::IdenticalIndices` | `index2 == index1` not allowed! |
    /// | `GraphErrors::EdgeDoesNotExist` | requested edge does not exists |
    pub fn remove_edge(&mut self, index1: u32, index2: u32) -> Result<(),GraphErrors> {
        let (r1, r2) = self.get_2_mut(index1, index2)?;
        r1.remove(r2)?;
        self.edge_count -= 1;
        Ok(())
    }

    /// returns total number of edges in graph
    pub fn edge_count(&self) -> u32 {
        self.edge_count
    }

    fn get_container(&self, index: usize) -> &NodeContainer<T> {
        &self.vertices[index]
    }

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
    pub fn dfs(&self, index: u32) -> Dfs<T> {
        Dfs::new(&self, index)
    }

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
    pub fn dfs_with_index(&self, index: u32) -> DfsWithIndex<T> {
        DfsWithIndex::new(&self, index)
    }

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
    pub fn bfs_index_depth(&self, index: u32) -> Bfs<T> {
        Bfs::new(&self, index)
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
            Some(self.dfs(0).count() == self.vertex_count() as usize)
        }
    }

    /// # definition
    /// Calculates the size of the **q-core** (i.e. number of nodes in the biggest possible set of nodes,
    /// where all nodes from the set are connected with at least `q` other nodes from the set)
    ///
    /// returns `None` if impossible to calculate (e.g. `vertex_count == 0` or `q <= 1`)
    /// # Example
    /// ```
    /// use net_ensembles::TestNode;
    /// use net_ensembles::Graph;
    ///
    /// let graph: Graph<TestNode> = Graph::new(0);
    /// assert_eq!(graph.q_core(1), None);
    /// assert_eq!(graph.q_core(2), None);
    ///
    /// let graph2: Graph<TestNode> = Graph::new(1);
    ///
    /// assert_eq!(graph2.q_core(1), None);
    /// assert_eq!(graph2.q_core(2), Some(0));
    ///
    ///
    /// // create complete graph
    /// let mut graph3: Graph<TestNode> = Graph::new(20);
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

    /// assert_eq!(graph3.q_core(20), Some(0));
    /// ```
    pub fn q_core(&self, q: u32) -> Option<u32> {
        if q < 2 || self.vertex_count() == 0 {
            return None;
        }
        let mut handled: Vec<bool> = vec![false; self.vertex_count() as usize];
        let mut subtract: Vec<usize> = vec![0; self.vertex_count() as usize];

        let q_usize = q as usize;
        let v_count = self.vertex_count() as usize;

        // virtually: recursively remove all vertices with less then q neighbors
        let mut something_changed = true;
        while something_changed {
            something_changed = false;
            for i in 0..v_count {
                if handled[i] {
                    continue;
                }

                // handle possible overflow
                let n_count = self
                    .get_container(i)
                    .neighbor_count();
                let remaining_neighbors = if subtract[i] >= n_count {
                    0
                } else {
                    n_count - subtract[i]
                };

                if remaining_neighbors < q_usize {
                    something_changed = true;

                    // virtually remove vertex
                    handled[i] = true;
                    for j in self.get_container(i).neighbors() {
                        subtract[*j as usize] += 1;
                    }
                }
            }
        }

        // find biggest component
        let mut result = 0;
        // initiate stack
        let mut stack: Vec<usize> = Vec::with_capacity(v_count);
        for i in 0..v_count {
            // skip all nodes that are removed or in a known component
            if handled[i] {
                continue;
            }
            let mut counter = 0;
            stack.push(i);
            handled[i] = true;
            while let Some(index) = stack.pop() {
                counter += 1;
                for j in self
                    .get_container(index)
                    .neighbors()    // iterate over neighbors
                    .map(|k| *k as usize) // but as usize
                {
                    // skip if already handled
                    if handled[j] {
                        continue;
                    }

                    handled[j] = true;
                    stack.push(j);
                }
            }
            result = max(result, counter);
        }

        Some(result)
    }

    /// # compute sizes of all *connected components*
    ///
    /// * the **number** of connected components is the **size** of the returned vector, i.e. `result.len()`
    /// * returns **empty** vector, if graph does not contain vertices
    /// * returns (reverse) **ordered vector of sizes** of the connected components,
    /// i.e. the biggest component is of size `result[0]` and the smallest is of size `result[result.len() - 1]`
    pub fn connected_components(&self) -> Vec<u32> {

        let mut component_id : Vec<i32> = vec![-1; self.vertex_count() as usize];
        let mut current_id = 0;

        for i in 0..self.vertex_count(){
            // already in a component?
            if component_id[i as usize] != -1 {
                continue;
            }

            // start depth first search over indices of vertices connected with vertex i
            for (j, _) in self.dfs_with_index(i) {
                component_id[j as usize] = current_id;
            }
            current_id += 1;

        }
        // cast current_id as usize
        let num_components = usize::try_from(current_id).ok()
            .expect("connected_components ERROR 0");

        let mut result = vec![0; num_components];

        for i in component_id {
            let as_usize = usize::try_from(i).ok()
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

    /// Count number of leaves in the graph, i.e. vertices with exactly one neighbor
    pub fn leaf_count(&self) -> usize {
        self.vertices
            .iter()
            .filter(|a| a.neighbor_count() == 1)
            .count()
    }

    /// * Creates String which contains the topology of the network in a format
    /// that can be used by **circo** etc. to generate a pdf of the graph.
    /// * **indices** are used as **labels**
    /// * search for **graphviz** to learn about **.dot** format
    pub fn to_dot(&self) -> String {
        let mut s = "graph{\n\t".to_string();

        for i in 0..self.vertex_count() {
            s += &format!("{} ", i);
        }
        s += "\n";

        for i in 0..self.vertex_count() as usize {
            for j in self.get_container(i).neighbors() {
                if i < *j as usize {
                    s.push_str(&format!("\t{} -- {}\n", i, j));
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
    /// use net_ensembles::{Graph,TestNode,DEFAULT_DOT_OPTIONS};
    ///
    /// let mut graph: Graph<TestNode> = Graph::new(3);
    /// graph.add_edge(0, 1).unwrap();
    /// graph.add_edge(0, 2).unwrap();
    /// graph.add_edge(1, 2).unwrap();
    ///
    /// // create string of dotfile
    /// let s = graph.to_dot_with_labels(
    ///    DEFAULT_DOT_OPTIONS,
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
    ///	    bgcolor="transparent";
    ///	    fontsize=50;
    ///	    node [shape=ellipse, penwidth=1, fontname="Courier", pin=true ];
    ///	    splines=true;
    ///	    0 1 2 ;
    ///	    "0" [label="Hey 0!"];
    ///	    "1" [label="Hey 1!"];
    ///	    "2" [label="Hey 2!"];
    ///	    0 -- 1
    ///	    0 -- 2
    ///	    1 -- 2
    /// }
    /// ```
    ///
    /// Then you can use, e.g.,
    /// ```console
    /// foo@bar:~$ circo example.dot -Tpdf > example.pdf
    /// ```
    /// to create a **pdf** representation from it.
    /// Search for **graphviz** to learn more.
    pub fn to_dot_with_labels<F>(&self, dot_options: &str, f: F ) -> String
        where F: Fn(&T, usize) -> String
    {
        let mut s = "graph G{\n\t"
                    .to_string();
        s += dot_options;
        s+= "\n\t";

        for i in 0..self.vertex_count() {
            s += &format!("{} ", i);
        }
        s += ";\n";
        for (index, vertex) in self.vertices.iter().enumerate() {
            s += &format!("\t\"{}\" [label=\"{}\"];\n", index, f(vertex.get_contained(), index));
        }

        for i in 0..self.vertex_count() as usize {
            for j in self.get_container(i).neighbors() {
                if i < *j as usize {
                    s.push_str(&format!("\t{} -- {}\n", i, j));
                }
            }
        }
        s += "}";
        s
    }

    /// * returns `None` **if** graph not connected **or** does not contain any vertices
    /// * uses repeated breadth first search
    pub fn diameter(&self) -> Option<u32> {
        if !self.is_connected()? {
            None
        } else {
            // well, then calculate from every node
            // (except 1 node) and use maximum foun
            self.node_container_iter()
            .skip(1)
            .map( |n|
                self.longest_shortest_path_from_index(n.get_id())
                    .expect("diameter ERROR 1")
            ).max()
        }
    }

    /// calculate the size of the longest shortest path **starting from** vertex with **index** `index`
    /// using breadth first search
    pub fn longest_shortest_path_from_index(&self, index: u32) -> Option<u32> {
        let iter = Bfs::new(&self, index);
        let (.., depth) = iter.last()?;
        Some(depth)
    }
}

/// # Breadth first search Iterator with **index** and **depth** of corresponding nodes
/// * iterator returns tuple: `(index, node, depth)`
pub struct Bfs<'a, T>
    where T: 'a + Node {
        graph: &'a Graph<T>,
        handled: Vec<bool>,
        queue0: VecDeque<u32>,
        queue1: VecDeque<u32>,
        depth: u32,
}

impl<'a, T> Bfs<'a, T>
    where T: 'a + Node {
        fn new(graph: &'a Graph<T>, index: u32) -> Self {
            let mut handled: Vec<bool> = vec![false; graph.vertex_count() as usize];
            let mut queue0 = VecDeque::with_capacity(graph.vertex_count() as usize);
            let queue1 = VecDeque::with_capacity(graph.vertex_count() as usize);
            let depth = 0;
            if index < graph.vertex_count() {
                queue0.push_back(index);
                handled[index as usize] = true;
            }
            let result = Bfs {
                graph,
                handled,
                queue0,
                queue1,
                depth,
            };
            result
        }
}

/// # Iterator
/// - returns tuple: `(index, node, depth)`
impl<'a, T> Iterator for Bfs<'a, T>
    where T: 'a + Node {
        type Item = (u32, &'a T, u32);
        fn next(&mut self) -> Option<Self::Item> {
            // if queue0 is not empty, take element from queue, push neighbors to other queue
            if let Some(index) = self.queue0.pop_front() {
                let container = self.graph.get_container(index as usize);
                for i in container.neighbors() {
                    if !self.handled[*i as usize] {
                        self.handled[*i as usize] = true;
                        self.queue1.push_back(*i);
                    }
                }
                Some((index, container.get_contained(), self.depth))
            }else if self.queue1.is_empty() {
                None
            }else {
                std::mem::swap(&mut self.queue0, &mut self.queue1);
                self.depth += 1;
                self.next()
            }
        }
}

/// Depth first search Iterator with **index** of corresponding nodes
pub struct DfsWithIndex<'a, T>
    where T: 'a + Node {
        graph: &'a Graph<T>,
        handled: Vec<bool>,
        stack: Vec<u32>,
}

impl<'a, T> DfsWithIndex<'a, T>
    where T: 'a + Node {
        fn new(graph: &'a Graph<T>, index: u32) -> Self {
            let mut handled: Vec<bool> = vec![false; graph.vertex_count() as usize];
            let mut stack: Vec<u32> = Vec::with_capacity(graph.vertex_count() as usize);
            if index < graph.vertex_count() {
                stack.push(index);
                handled[index as usize] = true;
            }

            DfsWithIndex {
                graph,
                handled,
                stack,
            }
        }

}

impl<'a, T> Iterator for DfsWithIndex<'a, T>
    where T: 'a + Node {
        type Item = (u32, &'a T);

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(index) = self.stack.pop(){
                let container = self.graph.get_container(index as usize);
                for i in container.neighbors() {
                    if !self.handled[*i as usize] {
                        self.handled[*i as usize] = true;
                        self.stack.push(*i);
                    }
                }
                Some((index, container.get_contained()))
            } else {
                None
            }
        }
}

/// Depth first search Iterator
pub struct Dfs<'a, T>
    where T: 'a + Node {
        graph: &'a Graph<T>,
        handled: Vec<bool>,
        stack: Vec<u32>,
}


impl<'a, T> Dfs<'a, T>
    where T: 'a + Node{
    /// panics if `index` >= graph.vertex_count()
    fn new(graph: &'a Graph<T>, index: u32) -> Self {
        let mut handled: Vec<bool> = vec![false; graph.vertex_count() as usize];
        let mut stack: Vec<u32> = Vec::with_capacity(graph.vertex_count() as usize);
        if index < graph.vertex_count() {
            stack.push(index);
            handled[index as usize] = true;
        }

        Dfs {
            graph,
            handled,
            stack,
        }
    }
}

impl<'a, T> Iterator for Dfs<'a, T>
    where T: 'a + Node {
        type Item = &'a T;

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(index) = self.stack.pop(){
                let container = self.graph.get_container(index as usize);
                for i in container.neighbors() {
                    if !self.handled[*i as usize] {
                        self.handled[*i as usize] = true;
                        self.stack.push(*i);
                    }
                }
                Some(container.get_contained())
            } else {
                None
            }
        }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::TestNode;


    #[test]
    fn test_graph_container_push() {
        // create two nodes
        let mut c = NodeContainer::new(0, TestNode::new_from_index(0));
        let mut c2 = NodeContainer::new(1, TestNode::new_from_index(1));
        // create edge -> should not result in error!
        let res = c.push(&mut c2);
        if let Err(e) = res {
            panic!(format!("error: {}", e));
        }
        // now edge exists, should not be able to add it again:
        let res = c.push(&mut c2);
        assert!(res.is_err());

        assert_eq!(0, c.get_id());
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

        println!("{}", graph.get_container(0));
    }

}
