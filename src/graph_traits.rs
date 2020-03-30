use crate::node::Node;
use std::fmt;
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


pub trait AdjContainer<T: Node> {
    fn parse_str(to_parse: &str) -> Option<(&str, Self)> where Self: Sized;
    fn contained<'a>(&'a self) -> &'a T;
    fn contained_mut(&mut self) -> &mut T;
    fn neighbors(&self) -> std::slice::Iter::<u32>;
    fn neighbor_count(&self) -> usize;
    fn id(&self) -> u32;
    fn is_adjacent(&self, other_id: &u32) -> bool;
}


pub trait GraphImpl<T: Node, N: AdjContainer<T>>
where
    T: Node,
    N: AdjContainer<T>,
    {
    type I1: Iterator<Item=N>;
    fn new(size: u32) -> Self;
    fn clear_edges(&mut self);
    fn sort_adj(&mut self);
    fn parse_str(to_parse: &str) -> Option<(&str, Self)> where Self: Sized;
    fn container(& self, index: usize) -> & N;
    fn container_iter(&self) -> std::slice::Iter::<N>;

    fn at(& self, index: usize) -> &T;
    fn at_mut(&mut self, index: usize) -> &mut T;
    fn vertex_count(&self) -> u32;
    fn edge_count(&self) -> u32;

    fn average_degree(&self) -> f32 {
        (2 * self.edge_count()) as f32 / self.vertex_count() as f32
    }

    fn add_edge(&mut self, index1: u32, index2: u32) -> Result<(),GraphErrors>;
    fn remove_edge(&mut self, index1: u32, index2: u32) -> Result<(),GraphErrors>;
    fn dfs(&self, index: u32) -> Self::I1;
}
