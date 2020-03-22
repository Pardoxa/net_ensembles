//! Trait needed for ER graphs etc.


/// What every node should be able to do
pub trait Node {
    /// how to construct a blank object 
    fn new_empty() -> Self;
}

/// minimal example for a node
pub struct TestNode {}


impl Node for TestNode {
    fn new_empty() -> Self {
        TestNode { }
    }
}

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn t1() {
        TestNode::new_empty();
    }
}
