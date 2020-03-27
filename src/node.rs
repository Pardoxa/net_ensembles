//! Trait needed for ER graphs etc.


/// What every node should be able to do
pub trait Node
where Self: Clone{
    /// how to construct a blank object
    fn new_from_index(index: u32) -> Self;

    /// Override this, if you want to store the network
    fn make_string(&self) -> Option<String> {
        None
    }

    /// Override this, if you want to load the stored network
    fn parse_str(_to_parse: &str) -> Option<(&str, Self)>
        where Self: Sized
    {
        None
    }
}

/// minimal example for a node
#[derive(Clone)]
pub struct TestNode {}


impl Node for TestNode {
    fn new_from_index(_: u32) -> Self {
        TestNode { }
    }
}

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn t1() {
        TestNode::new_from_index(1);
    }
}
