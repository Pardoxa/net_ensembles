use crate::IterWrapper;
use std::fmt;
/// What every node should be able to do
pub trait Node
where Self: Clone{
    /// how to construct a blank object
    fn new_from_index(index: u32) -> Self;

    /// Override this, if you want to store the network
    fn make_string(&self) -> Option<String> {
        None
    }

    /// Override this, if you want to load the stored network
    fn parse_str(_to_parse: &str) -> Option<(&str, Self)>
        where Self: Sized
    {
        None
    }
}



/// Error messages
#[derive(Debug)]
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

   pub(crate) fn to_sw_state(self) -> SwChangeState {
       SwChangeState::GError(self)
   }
}

impl fmt::Display for GraphErrors {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}


#[derive(Debug)]
pub enum SwChangeState {
    InvalidAdjecency,
    BlockedByExistingEdge,
    Nothing,
    /// old edge: (Rewire.0, Rewire.1), new edge (Rewire.0, Rewire.2)
    Rewire(u32, u32, u32),
    /// old edge: (Reset.0, Reset.1), new edge (Reset.0, Reset.2)
    Reset(u32, u32, u32),
    GError(GraphErrors),
}

pub trait AdjContainer<T: Node>
where   Self: fmt::Display,
{
    /// Create new instance with id
    fn new(id: u32, node: T) -> Self;

    /// # parse from str
    /// * tries to parse a AdjContainer from a `str`.
    /// * returns `None` if failed
    ///
    /// ## Returns `Option((a, b))`
    /// **a)** string slice beginning directly after the part, that was used to parse
    ///
    /// **b)** the `AdjContainer` resulting form the parsing
    fn parse_str(to_parse: &str) -> Option<(&str, Self)> where Self: Sized;

    /// return reference to what the AdjContainer contains
    fn contained<'a>(&'a self) -> &'a T;

    /// return mut reference to what the AdjContainer contains
    fn contained_mut(&mut self) -> &mut T;

    /// returns iterator over indices of neighbors
    fn neighbors<'b>(&self) -> IterWrapper;

    /// count number of neighbors, i.e. number of edges incident to `self`
    fn degree(&self) -> usize;

    /// returns id of container
    fn id(&self) -> u32;

    /// returns `Some(first element from the adjecency List)` or `None`
    fn get_adj_first(&self) -> Option<&u32>;

    /// check if vertex with `other_id` is adjacent to self
    /// ## Note:
    /// (in `Graph<T>`: `id` equals the index corresponding to `self`)
    fn is_adjacent(&self, other_id: &u32) -> bool;

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
