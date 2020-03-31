use crate::traits::*;
use std::fmt;
use crate::GraphErrors;

#[derive(Debug, Clone)]
struct SwEdge {
    to: u32,
    originally_to: Option<u32>,
}

impl SwEdge {
    fn to(&self) -> &u32 {
        &self.to
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
    /// (in `Graph<T>`: `id` equals the index corresponding to `self`)
    fn is_adjacent(&self, other_id: &u32) -> bool{
        self.neighbors()
            .any(|x| x == other_id)
    }

    /// Sorting adjecency lists
    fn sort_adj(&mut self){
        self.adj.sort_unstable_by(
            |a, b|
            a.to().partial_cmp(b.to()).unwrap()
        )
    }

    /// Remove all edges
    /// # Important
    /// * will not clear edges of other AdjContainer
    /// * only call this if you know exactly what you are doing
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

    fn swap_remove_element(&mut self, elem: u32) -> () {
        let index = self
            .neighbors()
            .position(|&x| x == elem)
            .expect("swap_remove_element ERROR 0");

        self.adj.swap_remove(index);
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
