//! # Created by Yannick Feld
//! My implementation for a graph
//!
use std::fmt;
use crate::node::Node;

/// all of my error messages
#[derive(Debug)]
pub enum GraphErrors{
    EdgeExists,
    EdgeDoesNotExist,
    IndexOutOfRange,
    IdenticalIndices,
}

impl GraphErrors {
   pub fn to_str(&self) -> &str {
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

#[allow(dead_code)]
struct GraphContainer<T>{
    id: u32,
    adj: Vec<u32>,
    node: T,
}

#[allow(dead_code)]
impl<T> GraphContainer<T> {
    pub fn new(id: u32, node: T) -> Self {
        GraphContainer{
            id,
            adj: Vec::new(),
            node,
        }
    }

    pub fn get_node(&self) -> &T {
        &self.node
    }

    pub fn neighbors(&self) -> std::slice::Iter::<u32> {
        self.adj.iter()
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn contains(&self, other_id: &u32) -> bool {
        self.adj.contains(other_id)
    }

    pub fn push(&mut self, other: &mut GraphContainer<T>) -> Result<(),GraphErrors> {
        if self.contains(&other.get_id()) {
            return Err(GraphErrors::EdgeExists);
        }
        self.adj.push(other.get_id());
        other.adj.push(self.id);
        Ok(())
    }

    fn swap_delete_element(&mut self, elem: u32) -> () {
        let index = self.adj.iter().position(|x| *x == elem).unwrap();
        self.adj.swap_remove(index);
    }

    /// Trys to remove edges, returns error `GraphErrors::EdgeDoesNotExist` if impossible
    pub fn remove(&mut self, other: &mut GraphContainer<T>) -> Result<(),GraphErrors> {
        if !self.contains(&other.get_id()){
            return Err(GraphErrors::EdgeDoesNotExist);
        }

        self.swap_delete_element(other.get_id());
        other.swap_delete_element(self.get_id());

        Ok(())
    }
}

#[allow(dead_code)]
pub struct Graph<T: Node> {
    vertices: Vec<GraphContainer<T>>,
    next_id: u32,
    edge_count: u32,
}

impl<T: Node> Graph<T> {
    /// Create new graph with `size` nodes
    /// and no edges
    pub fn new(size: u32) -> Self {
        let mut vertices = Vec::with_capacity(size as usize);
        for i in 0..size {
            let container = GraphContainer::new(i, T::new_empty());
            vertices.push(container);
        }
        Self{
            vertices,
            next_id: size,
            edge_count: 0,
        }
    }

    /// returns number of vertices present in graph
    pub fn vertex_count(&self) -> u32 {
        self.next_id
    }

    /// Returns two mutable references in a tuple
    /// ## ErrorCases:
    /// `GraphErrors::IndexOutOfRange`  <-- index to large
    /// GraphErrors::IdenticalIndices <-- index1 == index2 not allowed!
    fn get_2_mut(&mut self, index1: u32, index2: u32) ->
        Result<(&mut GraphContainer<T>, &mut GraphContainer<T>),GraphErrors>
    {
        if index1 >= self.next_id || index2 >= self.next_id {
            return Err(GraphErrors::IndexOutOfRange);
        } else if index1 == index2 {
            return Err(GraphErrors::IdenticalIndices);
        }
        let r1: &mut GraphContainer<T>;
        let r2: &mut GraphContainer<T>;

        let ptr = self.vertices.as_mut_ptr();

        unsafe {
            r1 = &mut *ptr.offset(index1 as isize);
            r2 = &mut *ptr.offset(index2 as isize);
        }

        Ok((r1, r2))
    }

    /// Adds edge between nodes `index1` and `index2`
    /// ## ErrorCases:
    /// | Error | Reason |
    /// | ---- | ---- |
    /// | `GraphErrors::IndexOutOfRange` | `index1` or `index2` larger than `self.vertex_count()`  |
    /// | `GraphErrors::IdenticalIndices` | `index2 == index1` not allowed! |
    /// | `GraphErrors::EdgeExists` | requested edge already exists! |
    pub fn add_edge(&mut self, index1: u32, index2: u32) -> Result<(),GraphErrors> {
        let (r1, r2) = self.get_2_mut(index1, index2)?;
        r1.push(r2)?;
        self.edge_count += 1;
        Ok(())
    }

    /// Removes edge between nodes *index1* and *index2*
    /// ## ErrorCases:
    /// | Error | Reason |
    /// | ---- | ---- |
    /// | `GraphErrors::IndexOutOfRange` | `index1` or `index2` larger than `self.vertex_count()`  |
    /// | `GraphErrors::IdenticalIndices` | `index2 == index1` not allowed! |
    /// | `GraphErrors::EdgeDoesNotExist` | requested edge does not exists |
    pub fn remove_edge(&mut self, index1: u32, index2: u32) -> Result<(),GraphErrors> {
        let (r1, r2) = self.get_2_mut(index1, index2)?;
        r1.remove(r2)?;
        self.edge_count -= 1;
        Ok(())
    }

    /// returns total number of edges in graph
    pub fn edge_count(&self) -> u32 {
        self.edge_count
    }

    fn get_container(&self, index: usize) -> &GraphContainer<T> {
        &self.vertices[index]
    }

    /// # returns Iterator
    ///
    /// the iterator will iterate over the vertices in depth first search order,
    /// beginning with vertex 0.
    ///
    /// Order
    ///------------------------
    /// Order is guaranteed to be in DFS order, however
    /// if this order is not unambigouse
    /// adding edges and especially removing edges will shuffle the order.
    ///
    /// Note:
    /// ----------------------
    /// Will only iterate over vertices within the connected component that contains vertex 0
    pub fn dfs(&self) -> Dfs<T> {
        Dfs::new(&self)
    }

    /// returns true if all vertices are connected by paths of edges, false otherwise
    pub fn is_connected(&self) -> bool {

        self.dfs().count() == self.vertex_count() as usize
    }
}

/// Depth first search Iterator
pub struct Dfs<'a, T>
    where T: 'a + Node {
    graph: &'a Graph<T>,
    handled: Vec<bool>,
    stack: Vec<u32>,
}


impl<'a, T> Dfs<'a, T>
    where T: 'a + Node{
    fn new(graph: &'a Graph<T>) -> Self {
        let mut handled: Vec<bool> = vec![false; graph.vertex_count() as usize];
        handled[0] = true;
        Dfs {
            graph,
            handled,
            stack: vec![0],
        }
    }
}

impl<'a, T> Iterator for Dfs<'a, T>
    where T: 'a + Node {
        type Item = &'a T;

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(index) = self.stack.pop(){
                let container = self.graph.get_container(index as usize);
                for i in container.neighbors() {
                    if !self.handled[*i as usize] {
                        self.handled[*i as usize] = true;
                        self.stack.push(*i);
                    }
                }
                Some(container.get_node())
            } else {
                None
            }
        }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_container() {
        let c = GraphContainer::new(0, 1);
        assert_eq!(0, c.get_id());
    }

    #[test]
    fn test_graph_container_push() {
        // create two nodes
        let mut c = GraphContainer::new(0, 1);
        let mut c2 = GraphContainer::new(1, 1);
        // create edge -> should not result in error!
        let res = c.push(&mut c2);
        if let Err(e) = res {
            panic!(format!("error: {}", e));
        }
        // now edge exists, should not be able to add it again:
        let res = c.push(&mut c2);
        assert!(res.is_err());

        assert_eq!(0, c.get_id());
    }
}
