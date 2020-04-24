//! Example nodes implementing trait `Node`
use crate::traits::*;

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

/// Use this, if you do not need to store extra information
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct EmptyNode {}

impl Node for EmptyNode {
    fn new_from_index(_: u32) -> Self {
        EmptyNode { }
    }
}
