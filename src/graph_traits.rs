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


pub trait AdjContainer<T: Node> where Self: fmt::Display {
    fn new(id: u32, node: T) -> Self;
    fn parse_str(to_parse: &str) -> Option<(&str, Self)> where Self: Sized;
    fn contained<'a>(&'a self) -> &'a T;
    fn contained_mut(&mut self) -> &mut T;
    fn neighbors(&self) -> std::slice::Iter::<u32>;
    fn neighbor_count(&self) -> usize;
    fn id(&self) -> u32;
    fn get_adj_first(&self) -> Option<&u32>;
    fn is_adjacent(&self, other_id: &u32) -> bool;

    fn sort_adj(&mut self);
    fn clear_edges(&mut self);
    fn push(&mut self, other: &mut Self)
        -> Result<(), GraphErrors>;
    fn remove(&mut self, other: &mut Self)
        -> Result<(), GraphErrors>;
}
