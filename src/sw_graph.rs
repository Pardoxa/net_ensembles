//! # Topology for SwEnsemble

use crate::traits::*;
use std::fmt;
use crate::GraphErrors;
use crate::SwChangeState;
use crate::GenericGraph;

#[derive(Debug, Clone)]
pub(crate) struct SwEdge {
    to: u32,
    originally_to: Option<u32>,
}

impl SwEdge {
    pub(crate) fn to(&self) -> &u32 {
        &self.to
    }

    fn originally_to(&self) -> &Option<u32> {
        &self.originally_to
    }

    pub(crate) fn is_root(&self) -> bool {
        self.originally_to.is_some()
    }

    fn reset(&mut self) {
        self.to = self.originally_to.unwrap();
    }

    fn rewire(&mut self, other_id: u32) {
        self.to = other_id;
    }

    fn is_at_root(&self) -> bool {
        self
            .originally_to
            .map_or(false, |val| val == self.to)
    }


    fn parse(to_parse: &str) -> Option<(&str, Self)> {
        // skip identifier PARSE_ID
        let mut split_index = to_parse.find("to: ")? + 4;
        let remaining_to_parse = &to_parse[split_index..];

        // find index of next PARSE_SEPERATOR and split there
        split_index = remaining_to_parse.find(",")?;
        let (to_str, remaining_to_parse) = remaining_to_parse.split_at(split_index);
        let to = to_str.parse::<u32>().ok()?;

        // skip identifier PARSE_ID
        split_index = remaining_to_parse.find("o: ")? + 3;
        let remaining_to_parse = &remaining_to_parse[split_index..];

        // find index of next PARSE_SEPERATOR and split there
        split_index = remaining_to_parse.find(".")?;
        let (root_str, remaining_to_parse) = remaining_to_parse.split_at(split_index);

        let mut char_iter = root_str.chars();
        let originally_to = if Some('S') == char_iter.next() {
            let orig = char_iter.as_str().parse::<u32>().ok()?;
            Some(orig)
        } else {
            None
        };

        Some((&remaining_to_parse[1..], SwEdge{to, originally_to}))
    }
}

impl fmt::Display for SwEdge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // "id: {} adj: {:?} Node: {}"
        let originally_to = match self.originally_to {
            None => "N".to_string(),
            Some(o) => format!("S{}", o),
        };
        write!(f, "to: {}, o: {}.", self.to, originally_to)
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
/// * contains Adjacency list
///  and internal id (normally the index in the graph).
/// * also contains user specified data, i.e, `T` from `SwContainer<T>`
/// * see trait **`AdjContainer`**
#[derive(Debug, Clone)]
pub struct SwContainer<T: Node> {
    id: u32,
    adj: Vec<SwEdge>,
    node: T,
}


impl<T: Node> fmt::Display for SwContainer<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut adj_str_list = Vec::with_capacity(self.adj.len());
        for e in self.adj.iter() {
            adj_str_list.push(e.to_string());
        }
        let adj_str = adj_str_list.join(" ");

        let node_s = self.node.make_string()
            .expect(&format!("make_string failed - \
                Did you forget to Override? \
                Look at {}::Node", env!("CARGO_PKG_NAME")));
        write!(
            f,
            "id: {}, Node: {} adj: {}[{}]",
            self.id,
            node_s,
            self.adj.len(),
            adj_str
        )
    }
}

impl<T: Node> AdjContainer<T> for SwContainer<T>
{
    /// Create new instance with id
    fn new(id: u32, node: T) -> Self {
        SwContainer{
            id,
            node,
            adj: Vec::new(),
        }
    }

    /// # parse from str
    /// * tries to parse a AdjContainer from a `str`.
    /// * returns `None` if failed
    ///
    /// ## Returns `Option((a, b))`
    /// **a)** string slice beginning directly after the part, that was used to parse
    ///
    /// **b)** the `AdjContainer` resulting form the parsing
    fn parse_str(to_parse: &str) -> Option<(&str, Self)> where Self: Sized{
        // skip identifier
        let mut split_index = to_parse.find("id: ")? + 4;
        let remaining_to_parse = &to_parse[split_index..];

        // find index of next PARSE_SEPERATOR and split there
        split_index = remaining_to_parse.find(",")?;
        let (id_str, mut remaining_to_parse) = remaining_to_parse.split_at(split_index);
        let id = id_str.parse::<u32>().ok()?;

        // parse Node
        split_index = remaining_to_parse.find("Node: ")? + 6;
        remaining_to_parse = &remaining_to_parse[split_index..];

        let (mut remaining_to_parse, node) = T::parse_str(remaining_to_parse)?;

        // parse adj
        split_index = remaining_to_parse.find("adj: ")? + 5;
        remaining_to_parse = &remaining_to_parse[split_index..];

        // how large is adj?
        split_index = remaining_to_parse.find("[")?;
        let (len_str, mut remaining_to_parse) = remaining_to_parse.split_at(split_index);

        let len = len_str.parse::<usize>().ok()?;

        let mut adj = Vec::with_capacity(len);
        for _ in 0..len {
            let result = SwEdge::parse(remaining_to_parse)?;
            remaining_to_parse = result.0;
            adj.push(result.1);
        }
        Some((
                remaining_to_parse,
                Self{
                    id,
                    node,
                    adj,
                }
            ))
    }

    /// return reference to what the AdjContainer contains
    fn contained<'a>(&'a self) -> &'a T{
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
    fn id(&self) -> u32{
        self.id
    }

    /// returns `Some(first element from the adjecency List)` or `None`
    fn get_adj_first(&self) -> Option<&u32>{
        Some (
            self.adj
                .first()?
                .to()
        )
    }

    /// check if vertex with `other_id` is adjacent to self
    /// ## Note:
    /// (in `GenericGraph<T>`: `id` equals the index corresponding to `self`)
    fn is_adjacent(&self, other_id: &u32) -> bool{
        SwEdgeIterNeighbors::new(self.adj.as_slice())
            .any(|x| x == other_id)
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
            if self.is_adjacent(&other.id()) {
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
            if !self.is_adjacent(&other.id()){
                return Err(GraphErrors::EdgeDoesNotExist);
            }
            self.swap_remove_element(other.id());
            other.swap_remove_element(self.id());

            Ok(())
        }
}

impl<T: Node> SwContainer<T> {

    fn adj_position(&self, elem: u32) -> Option<usize> {
        SwEdgeIterNeighbors::new(self.adj.as_slice())
            .position(|&x| x == elem)
    }

    fn swap_remove_element(&mut self, elem: u32) -> () {
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
    unsafe fn push_single(&mut self, other_id: u32) {
        debug_assert!(
            !self.is_adjacent(&other_id),
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
        else if self.is_adjacent(&to_rewire.id())
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
    fn reset(&mut self, other: &mut Self) -> Result<u32, SwChangeState> {

        let self_index = self.adj_position(other.id());

        if self_index.is_none() {
            return Err(GraphErrors::EdgeDoesNotExist.to_sw_state());
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
            &self_edge
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

impl<T: Node> SwGraph<T>{
    /// # Reset small-world edge to its root state
    /// * **panics** if index out of bounds
    /// * in debug: panics if `index0 == index1`
    pub fn reset_edge(&mut self, index0: u32, index1: u32) -> SwChangeState {
        let (e1, e2) = self.get_2_mut(index0, index1);

        let vertex_index =
            match e1.reset(e2) {
                Err(error) => return error,
                Ok(container_tuple) => container_tuple
            };

        unsafe {
            self
                .get_mut_unchecked(vertex_index as usize)
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
    pub fn rewire_edge(&mut self, index0: u32, index1: u32, index2: u32) -> SwChangeState {
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
        self.clear_edges();
        let n = self.vertex_count();

        for i in 0..n {
            self.add_edge(i, (i + 1) % n)
                .unwrap();
            self.add_edge(i, (i + 2) % n)
                .unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use rand::Rng;
    use rand_pcg::Pcg64;
    use rand::SeedableRng;

    #[test]
    fn sw_ring_2() {
        let size = 300u32;
        let mut graph = SwGraph::<EmptyNode>::new(size);
        graph.init_ring_2();

        assert_eq!(graph.vertex_count(), size);
        assert_eq!(graph.edge_count(), size * 2);

        for i in 0..size {
            assert_eq!(2, graph.container(i as usize).count_root());
        }

        graph.sort_adj();

        for i in 0..size {
            let container = graph.container(i as usize);
            assert_eq!(container.id(), i);
            let m2 = if i >= 2 {i - 2} else { (i + size - 2) % size };
            let m1 = if i >= 1 {i - 1} else { (i + size - 1) % size };
            let p1 = (i + 1) % size;
            let p2 = (i + 2) % size;
            let mut v = vec![(m2, None), (m1, None), (p1, Some(p1)), (p2, Some(p2))];
            v.sort_unstable_by_key(|x| x.0);

            let iter = graph
                .get_mut_unchecked(i as usize)
                .iter_raw_edges();
            for (edge, other_edge) in iter.zip(v.iter()) {
                assert_eq!(edge.to(), &other_edge.0);
            }
        }

    }

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
                to: rng.gen::<u32>(),
                originally_to: o,
            };

            let s = format!("{}", e);
            let (_, parsed) = SwEdge::parse(&s).unwrap();
            assert_eq!(parsed.to, e.to);
            assert_eq!(parsed.originally_to, e.originally_to);
        }
    }

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
            let s = c.to_string();
            let (_, parsed) = SwContainer::<EmptyNode>::parse_str(&s).unwrap();
            assert_eq!(parsed.degree(), c.degree());
            assert_eq!(parsed.id(), c.id());
            for (i, j) in c.adj.iter().zip(parsed.adj.iter()){
                assert_eq!(i.to, j.to);
                assert_eq!(i.originally_to, j.originally_to);
            }
        }
    }
}
