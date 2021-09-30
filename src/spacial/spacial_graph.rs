//! # Topology
//! Implements a network with x and y coordinate.
//!
//! You probably want to take a look at the struct `GenericGraph`,

use crate::{traits::*, GraphErrors, GenericGraph};

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

/// # Used for accessing neighbor information from graph
/// * contains Adjacency list
///  and internal id (normally the index in the graph).
/// * also contains user specified data, i.e, `T` from `NodeContainer<T>`
/// * see trait **`AdjContainer`**
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct SpacialNodeContainer<T>
{
    pub(crate) adj: Vec<usize>,
    id: usize,
    pub(crate) x: f64,
    pub(crate) y: f64,
    node: T,
}


impl<T: Node + SerdeStateConform> AdjContainer<T> for SpacialNodeContainer<T> {

    /// Create new instance with id
    fn new(id: usize, node: T) -> Self {
        SpacialNodeContainer{
            id,
            adj: Vec::new(),
            node,
            x: f64::NAN,
            y: f64::NAN,
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

impl<T> SpacialNodeContainer<T> {

    fn swap_remove_element(&mut self, elem: usize) {
        let index = self.adj
            .iter()
            .position(|&x| x == elem)
            .expect("swap_remove_element ERROR 0");

        self.adj
            .swap_remove(index);
    }

    /// Calculates the distance between two nodes
    ///
    /// Assumes that both nodes have valid x and y coordinates
    pub fn distance(&self, other: &Self) -> f64 {
        let x = self.x - other.x;
        let y = self.y - other.y;
        y.hypot(x)
    }
}

/// Type definiton for convinience. This is used to implement 
/// the spacial ensemble
pub type SpacialGraph<T> = GenericGraph<T, SpacialNodeContainer<T>>;


impl<T> SpacialGraph<T>
{
    /// # Euclidean distance between two vertices
    /// * Calculates the distance between the vertices 
    /// corresponding to the indices `i` and `j`
    /// * `None` if any of the indices is out of bounds
    pub fn distance(&self, i: usize, j: usize) -> Option<f64>
    where SpacialNodeContainer<T>: AdjContainer<T>
    {
        let container_i = self.container_checked(i)?;
        
        self.container_checked(j)
            .map(
                |container_j|
                {
                    container_i.distance(container_j)
                }
            )
    }
}