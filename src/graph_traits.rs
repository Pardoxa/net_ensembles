use crate::IterWrapper;
use std::fmt;
use crate::sw::SwChangeState;
use crate::traits::SerdeStateConform;
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
