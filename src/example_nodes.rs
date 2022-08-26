//! Example nodes implementing trait `Node`
use crate::traits::*;

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

/// Use this, if you do not need to store extra information
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct EmptyNode {}

impl Node for EmptyNode {
    fn new_from_index(_: usize) -> Self {
        EmptyNode { }
    }
}

/// Example node that contains a `usize` which will, if not changed, 
/// correspond to the index of the Vertex
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct CountingNode {
    /// Contained data
    pub index: usize
}

impl CountingNode {
    /// Returns the index of the node
    pub fn number(&self) -> usize
    {
        self.index
    }
}

impl Node for CountingNode {
    fn new_from_index(index: usize) -> Self
    {
        Self{index}
    }
}