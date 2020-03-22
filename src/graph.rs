use std::fmt;
use crate::node::Node;

pub enum GraphErrors{
    EdgeExistsAllready,
    EdgeDoesNotExist,
    IndexOutOfRange,
    IdenticalIndices,
}

impl GraphErrors {
   pub fn to_str(&self) -> &str {
       match self {
           GraphErrors::EdgeExistsAllready  => &"EdgeExistsAllready",
           GraphErrors::EdgeDoesNotExist    => &"EdgeDoesNotExist",
           GraphErrors::IndexOutOfRange     => &"IndexOutOfRange",
           GraphErrors::IdenticalIndices  => &"IdenticalIndices",
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

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn contains(&self, other_id: &u32) -> bool {
        self.adj.contains(other_id)
    }

    pub fn push(&mut self, other: &mut GraphContainer<T>) -> Result<(),GraphErrors> {
        if self.contains(&other.get_id()) {
            return Err(GraphErrors::EdgeExistsAllready);
        }
        self.adj.push(other.get_id());
        other.adj.push(self.id);
        Ok(())
    }

    pub fn remove(&mut self, other: &mut GraphContainer<T>) -> Result<(),GraphErrors> {
        if !self.contains(&other.get_id()){
            return Err(GraphErrors::EdgeDoesNotExist);
        }
        Ok(())
    }
}

#[allow(dead_code)]
pub struct Graph<T: Node> {
    vertices: Vec<GraphContainer<T>>,
    next_id: u32,
}

impl<T: Node> Graph<T> {
    pub fn new(size: u32) -> Self {
        let mut vertices = Vec::with_capacity(size as usize);
        for i in 0..size {
            let container = GraphContainer::new(i, T::new_empty());
            vertices.push(container);
        }
        Self{
            vertices,
            next_id: size,
        }
    }

    pub fn size(&self) -> u32 {
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
            let p1 = ptr.offset(index1 as isize);
            let p2 = ptr.offset(index2 as isize);
            r1 = &mut *p1;
            r2 = &mut *p2;
        }

        Ok((r1, r2))
    }

    /// Adds edge between nodes *index1* and *index2*
    /// ## ErrorCases:
    /// | Error | Reason |
    /// | ---- | ---- |
    /// | `GraphErrors::IndexOutOfRange` | index to large  |
    /// | `GraphErrors::IdenticalIndices` | index2 == index1 not allowed! |
    pub fn add_edge(&mut self, index1: u32, index2: u32) -> Result<(),GraphErrors> {
        let (r1, r2) = self.get_2_mut(index1, index2)?;
        r1.push(r2)?;
        Ok(())
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
