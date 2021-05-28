//! # Topology for SwEnsemble

use crate::{step_structs::*, GenericGraph, traits::*};
use core::iter::FusedIterator;
use std::vec;

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

/// # Edge of small world network
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct SwEdge {
    to: usize,
    originally_to: Option<usize>,
}

impl SwEdge {
    /// Where does the edge point to, i.e., to which node does it connect?
    pub fn to(&self) -> &usize {
        &self.to
    }

    pub(crate) fn originally_to(&self) -> &Option<usize> {
        &self.originally_to
    }

    /// # Is the edge a root edge?
    /// A root edge is an edge which will allways be connected to the current node.
    /// Where it connects to can change, where it connects from cannot.
    pub fn is_root(&self) -> bool {
        self.originally_to.is_some()
    }

    fn reset(&mut self) {
        self.to = self.originally_to.unwrap();
    }

    fn rewire(&mut self, other_id: usize) {
        self.to = other_id;
    }

    /// # Is the edge at its root position?
    /// A root edge is an edge which will allways be connected to the current node.
    /// Where it connects to can change, where it connects from cannot.
    /// 
    /// A root edge can be reset (i.e., where it connects to is reset) to its original neighbor
    /// (the next, or second next neighbor)
    ///
    /// This checks, if the edge still connects to where it would be reset to anyway
    pub fn is_at_root(&self) -> bool {
        self
            .originally_to
            .map_or(false, |val| val == self.to)
    }

    /// # checks root edge it it is long ranging
    /// * is it a root edge and if yes, is it a long ranging root edge?
    pub fn is_long_ranging_root(&self) -> bool {
        self.originally_to
            .map_or(false, |val| val != self.to)
    }

}

/// Iterator over indices stored in adjecency list
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

impl<'a> FusedIterator for SwEdgeIterNeighbors<'a> {}

impl<'a> Iterator for SwEdgeIterNeighbors<'a> {
    type Item = &'a usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {

        let (edge, next_slice) = self
            .sw_edge_slice
            .split_first()?;
        self.sw_edge_slice = next_slice;
        Some(edge.to())
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.sw_edge_slice.len() {
            self.sw_edge_slice = &[];
            None
        } else{
            let (elements, next_slice) = self
                .sw_edge_slice
                .split_at(n + 1);
            self.sw_edge_slice = next_slice;

            Some(elements[n].to())
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a> DoubleEndedIterator for SwEdgeIterNeighbors<'a> {

    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let (edge, next_slice) = self
            .sw_edge_slice
            .split_last()?;
        self.sw_edge_slice = next_slice;
        Some(edge.to())
    }
}

impl<'a> ExactSizeIterator for SwEdgeIterNeighbors<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.sw_edge_slice.len()
    }
}

/// # Used for accessing neighbor information from graph
/// * contains Adjacency list
///  and internal id (normally the index in the graph).
/// * also contains user specified data, i.e, `T` from `SwContainer<T>`
/// * see trait **`AdjContainer`**
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct SwContainer<T: Node> {
    id: usize,
    adj: Vec<SwEdge>,
    node: T,
}

impl<T> AdjList<SwEdge> for SwContainer<T>
where T: Node
{
    fn edges(&self) -> &[SwEdge] {
        &self.adj
    }
}

impl<T> AdjContainer<T> for SwContainer<T>
where T: Node + SerdeStateConform
{
    /// Create new instance with id
    fn new(id: usize, node: T) -> Self {
        SwContainer{
            id,
            node,
            adj: Vec::new(),
        }
    }

    /// return reference to what the AdjContainer contains
    fn contained(&self) -> & T{
        &self.node
    }

    /// return mut reference to what the AdjContainer contains
    fn contained_mut(&mut self) -> &mut T{
        &mut self.node
    }

    /// returns iterator over indices of neighbors
    fn neighbors(&self) -> IterWrapper {
        IterWrapper::new_sw(
            SwEdgeIterNeighbors::new(self.adj.as_slice())
        )
    }

    /// count number of neighbors, i.e. number of edges incident to `self`
    fn degree(&self) -> usize{
        self.adj.len()
    }

    /// returns id of container
    fn id(&self) -> usize {
        self.id
    }

    /// returns `Some(first element from the adjecency List)` or `None`
    fn get_adj_first(&self) -> Option<&usize>{
        Some (
            self.adj
                .first()?
                .to()
        )
    }

    /// check if vertex with `other_id` is adjacent to self
    /// ## Note:
    /// (in `GenericGraph<T>`: `id` equals the index corresponding to `self`)
    fn is_adjacent(&self, other_id: usize) -> bool{
        SwEdgeIterNeighbors::new(self.adj.as_slice())
            .any(|&x| x == other_id)
    }

    /// # Sorting adjecency lists
    /// * worst case: `O(edges log(edges))`
    fn sort_adj(&mut self){
        self.adj.sort_unstable_by_key(
            |k| *k.to()
        )
    }

    /// Remove all edges
    /// # Important
    /// * will not clear edges of other AdjContainer
    /// * only call this if you know exactly what you are doing
    #[doc(hidden)]
    unsafe fn clear_edges(&mut self){
        self.adj.clear();
    }

    /// # What does it do?
    /// * creates edge in `self` and `other`s adjecency Lists
    /// * edge root is set to self
    /// # Why is it unsafe?
    /// * No logic to see, if AdjContainer are part of the same graph
    /// * Only intended for internal usage
    /// ## What should I do?
    /// * use members of `net_ensembles::GenericGraph` instead, that handles the logic
    #[doc(hidden)]
    unsafe fn push(&mut self, other: &mut Self)
        -> Result<(), GraphErrors>{
            if self.is_adjacent(other.id()) {
                return Err(GraphErrors::EdgeExists);
            }

            // push the root link
            let root_link = SwEdge{
                to: other.id(),
                originally_to: Some(other.id()),
            };
            self.adj.push(root_link);

            // push the loose link
            let loose_link = SwEdge {
                to: self.id(),
                originally_to: None,
            };
            other.adj.push(loose_link);
            Ok(())
        }

    /// # What does it do?
    /// Removes edge in `self` and `other`s adjecency Lists
    /// # Why is it unsafe?
    /// * No logic to see, if AdjContainer are part of the same graph
    /// * Only intended for internal usage
    /// ## What should I do?
    /// * use members of `net_ensembles::GenericGraph` instead, that handles the logic
    #[doc(hidden)]
    unsafe fn remove(&mut self, other: &mut Self)
        -> Result<(), GraphErrors>{
            if !self.is_adjacent(other.id()){
                return Err(GraphErrors::EdgeDoesNotExist);
            }
            self.swap_remove_element(other.id());
            other.swap_remove_element(self.id());

            Ok(())
        }
}

impl<T: Node + SerdeStateConform> SwContainer<T> {

     /// returns iterator over indices of neighbors
     /// * Iterator returns the same items as `self.neigbors()`, though
     /// it might be more efficient. It will never be less efficient
     pub fn neighbors_sw(&self) -> SwEdgeIterNeighbors {
        SwEdgeIterNeighbors::new(self.adj.as_slice())
    }

    fn adj_position(&self, elem: usize) -> Option<usize> {
        SwEdgeIterNeighbors::new(self.adj.as_slice())
            .position(|&x| x == elem)
    }

    fn swap_remove_element(&mut self, elem: usize) {
        let index = self.adj_position(elem)
            .expect("swap_remove_element ERROR 0");

        self.adj.swap_remove(index);
    }

    /// Count how many root edges are contained
    pub fn count_root(&self) -> usize {
        self.adj
        .iter()
        .filter(|edge| edge.is_root())
        .count()
    }

    /// Iterate over the concrete edge impl
    #[allow(dead_code)]
    pub(crate) fn iter_raw_edges(&self) -> std::slice::Iter::<SwEdge> {
        self.adj.iter()
    }

    /// # Add something like an "directed" edge
    /// * unsafe, if edge exists already
    /// * panics in debug, if edge exists already
    /// * intended for usage after `reset`
    /// * No guarantees whatsoever, if you use it for something else
    unsafe fn push_single(&mut self, other_id: usize) {
        debug_assert!(
            !self.is_adjacent(other_id),
            "SwContainer::push single - ERROR: pushed existing edge!"
        );
        self.adj
            .push( SwEdge{ to: other_id, originally_to: None } );
    }

    fn rewire(&mut self, to_disconnect: &mut Self, to_rewire: &mut Self) -> SwChangeState {
        // None if edge does not exist
        let self_edge_index = self
            .adj_position(to_disconnect.id());

        // check if rewire request is invalid
        if self_edge_index.is_none()
        {
            return SwChangeState::InvalidAdjecency;
        }
        else if self.is_adjacent(to_rewire.id())
        {
            return SwChangeState::BlockedByExistingEdge;
        }
        let self_edge_index = self_edge_index.unwrap();

        debug_assert!(
            self.adj[self_edge_index].is_root(),
            "rewire - edge (self, to_rewire) has to be rooted at self!"
        );

        // remove edge
        to_disconnect.adj
            .swap_remove(
                to_disconnect
                .adj_position(self.id())
                .unwrap()
            );

        // rewire root
        self.adj[self_edge_index]
            .rewire(to_rewire.id());

        // add corresponding edge
        to_rewire.adj
            .push( SwEdge{ to: self.id(), originally_to: None} );

        SwChangeState::Rewire(
            self.id(),
            to_disconnect.id(),
            to_rewire.id()
        )
    }

    /// # Intendet to reset a small world edge
    /// * If successful will return edge, that needs to be added to the graph **using `push_single`**
    /// # panics
    /// * in debug: if edge not rooted at self
    fn reset(&mut self, other: &mut Self) -> Result<usize, SwChangeState> {

        let self_index = self.adj_position(other.id());

        if self_index.is_none() {
            return Err(GraphErrors::EdgeDoesNotExist.convert_to_sw_state());
        }

        let other_index = other.adj_position(self.id());
        debug_assert!(other_index.is_some());

        let self_index = self_index.expect("ERROR 0 SwContainer reset");
        let other_index = other_index.expect("ERROR 1 SwContainer reset");

        // assert safty for get_unchecked
        assert!(self_index < self.adj.len());
        assert!(other_index < other.adj.len());

        let self_edge  = unsafe{ self. adj.get_unchecked(self_index) };
        let other_edge = unsafe{ other.adj.get_unchecked(other_index) };

        debug_assert!(
            !other_edge.is_root(),
            "Error in reset: edge has to be rooted at self"
        );

        // if edge is allready at root, there is nothing to be done
        if self_edge.is_at_root() {
            Err(SwChangeState::Nothing)
        }
        // check if edge exists
        else if self.is_adjacent(
            self_edge
                .originally_to()
                .unwrap()
            ){
            // edge already exists!
            Err(SwChangeState::BlockedByExistingEdge)
        } else {
            // self is root edge, reset it, remove other
            self.adj[self_index].reset();
            other.adj.swap_remove(other_index);
            Ok(*self.adj[self_index].to())
        }
    }
}

/// specific `GenericGraph` used for small-world ensemble
pub type SwGraph<T> = GenericGraph<T, SwContainer<T>>;

impl<T> SwGraph<T>
where T: Node + SerdeStateConform {
    /// # Reset small-world edge to its root state
    /// * **panics** if index out of bounds
    /// * in debug: panics if `index0 == index1`
    pub fn reset_edge(&mut self, index0: usize, index1: usize) -> SwChangeState {
        let (e1, e2) = self.get_2_mut(index0, index1);

        let vertex_index =
            match e1.reset(e2) {
                Err(error) => return error,
                Ok(container_tuple) => container_tuple
            };

        unsafe {
            self
                .get_mut_unchecked(vertex_index)
                .push_single(index0);
        }
        SwChangeState::Reset(index0, index1, vertex_index)
    }

    /// # Rewire edges
    /// * rewire edge `(index0, index1)` to `(index0, index2)`
    /// # panics
    /// *  if indices are out of bounds
    /// *  in debug: panics if `index0 == index2`
    /// *  edge `(index0, index1)` has to be rooted at `index0`, else will panic in **debug** mode
    pub fn rewire_edge(&mut self, index0: usize, index1: usize, index2: usize) -> SwChangeState {
        if index1 == index2 {
            return SwChangeState::Nothing;
        }
        let (c0, c1, c2) = self.get_3_mut(index0, index1, index2);

        c0.rewire(c1, c2)
    }

    /// # initialize Ring2
    /// * every node is connected with its
    /// next and second next neighbors
    pub(crate) fn init_ring_2(&mut self) {
        self.init_ring(2)
            .expect("unable to init Ring");
    }


    /// # How many nodes have long ranging edges?
    /// * counts how many nodes have long ranging edges
    pub fn count_nodes_with_long_ranging_edges(&self) -> usize
    {
        let mut has_long_ranging_edge = vec![false; self.vertex_count()];
        self.container_iter()
            .enumerate()
            .for_each(
                |(index, c)|
                c.iter_raw_edges()
                    .filter(|e| e.is_long_ranging_root())
                    .for_each(
                        |e|
                        {
                            has_long_ranging_edge[index] = true;
                            has_long_ranging_edge[*e.to()] = true;
                        }
                    )
            );

        has_long_ranging_edge.into_iter()
            .filter(|long_ranging_edge| *long_ranging_edge)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[cfg(feature = "serde_support")]
    use rand::Rng;
    #[cfg(feature = "serde_support")]
    use rand_pcg::Pcg64;
    #[cfg(feature = "serde_support")]
    use rand::SeedableRng;
    #[cfg(feature = "serde_support")]
    use serde_json;

    #[test]
    fn sw_ring_2() {
        let size = 300;
        let mut graph = SwGraph::<EmptyNode>::new(size);
        graph.init_ring_2();

        assert_eq!(graph.vertex_count(), size);
        assert_eq!(graph.edge_count(), size * 2);

        for i in 0..size {
            assert_eq!(2, graph.container(i).count_root());
        }

        graph.sort_adj();

        for i in 0..size {
            let container = graph.container(i);
            assert_eq!(container.id(), i);
            let m2 = if i >= 2 {i - 2} else { (i + size - 2) % size };
            let m1 = if i >= 1 {i - 1} else { (i + size - 1) % size };
            let p1 = (i + 1) % size;
            let p2 = (i + 2) % size;
            let mut v = vec![(m2, None), (m1, None), (p1, Some(p1)), (p2, Some(p2))];
            v.sort_unstable_by_key(|x| x.0);

            let iter = graph
                .get_mut_unchecked(i)
                .iter_raw_edges();
            for (edge, other_edge) in iter.zip(v.iter()) {
                assert_eq!(edge.to(), &other_edge.0);
            }
        }

    }

    #[cfg(feature = "serde_support")]
    #[test]
    fn sw_edge_parse(){
        let mut rng = Pcg64::seed_from_u64(45767879234332);
        for _ in 0..2000 {
            let o = if rng.gen::<f64>() < 0.5 {
                Some(rng.gen())
            }else{
                None
            };
            let e = SwEdge{
                to: rng.gen::<usize>(),
                originally_to: o,
            };

            let s = serde_json::to_string(&e).unwrap();
            let parsed: SwEdge = serde_json::from_str(&s).unwrap();
            assert_eq!(parsed.to, e.to);
            assert_eq!(parsed.originally_to, e.originally_to);
        }
    }

    #[cfg(feature = "serde_support")]
    #[test]
    fn sw_container_test() {
        let mut c1 = SwContainer::new(0, EmptyNode{});
        let mut c2 = SwContainer::new(1, EmptyNode{});
        let mut c3 = SwContainer::new(1, EmptyNode{});

        unsafe  {
            c1.push(&mut c2).ok().unwrap();
            c2.push(&mut c3).ok().unwrap();
            c3.push(&mut c1).ok().unwrap();
        }
        let v = vec![c1, c2, c3];
        for c in v {
            let s = serde_json::to_string(&c).unwrap();
            let parsed: SwContainer::<EmptyNode> = serde_json::from_str(&s).unwrap();
            assert_eq!(parsed.degree(), c.degree());
            assert_eq!(parsed.id(), c.id());
            for (i, j) in c.adj.iter().zip(parsed.adj.iter()){
                assert_eq!(i.to, j.to);
                assert_eq!(i.originally_to, j.originally_to);
            }
        }
    }
}
