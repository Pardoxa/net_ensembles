use crate::traits::*;
use std::fmt;

#[derive(Debug, Clone)]
struct SwEdge {
    to: u32,
    originally_to: Option<u32>,
}

impl SwEdge {
    fn to(&self) -> &u32 {
        &self.to
    }
}

pub struct SwEdgeIterNeighbors<'a> {
    sw_edge_slice: &'a[SwEdge],
}

impl<'a> SwEdgeIterNeighbors<'a> {
    fn new(sw_edge_slice: &'a[SwEdge]) -> Self {
        Self {
            sw_edge_slice,
        }
    }
}

impl<'a> Iterator for SwEdgeIterNeighbors<'a> {
    type Item = &'a u32;
    fn next(&mut self) -> Option<Self::Item> {

        let (edge, next_slice) = self
            .sw_edge_slice
            .split_first()?;
        self.sw_edge_slice = next_slice;
        Some(edge.to())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a> DoubleEndedIterator for SwEdgeIterNeighbors<'a> {

    fn next_back(&mut self) -> Option<Self::Item> {
        let (edge, next_slice) = self
            .sw_edge_slice
            .split_last()?;
        self.sw_edge_slice = next_slice;
        Some(edge.to())
    }
}

impl<'a> ExactSizeIterator for SwEdgeIterNeighbors<'a> {
    fn len(&self) -> usize {
        self.sw_edge_slice.len()
    }
}

/// # Used for accessing neighbor information from graph
/// * Contains Adjacency list
///  and internal id. Normally the index in the graph.
/// * Also contains user specified data, i.e, `T` from `NodeContainer<T>`
/// * See trait **`AdjContainer`**
#[derive(Debug, Clone)]
pub struct SwContainer<T: Node>{
    id: u32,
    adj: Vec<SwEdge>,
    node: T,
}


impl<T: Node> fmt::Display for SwContainer<T> {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        // "id: {} adj: {:?} Node: {}"
        unimplemented!();
    }
}


impl<T: Node> SwContainer<T> {
    pub fn neighbors(&self) -> IterWrapper {
        IterWrapper::new_sw(
            SwEdgeIterNeighbors::new(self.adj.as_slice())
        )
    }
}
