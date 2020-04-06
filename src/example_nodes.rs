//! Example nodes implementing trait `Node`
use crate::traits::*;

/// minimal example for a node
#[derive(Clone)]
pub struct TestNode {}


impl Node for TestNode {
    fn new_from_index(_: u32) -> Self {
        TestNode { }
    }
}

/// Use this, if you do not need to store extra information
#[derive(Clone, Debug)]
pub struct EmptyNode {}

impl Node for EmptyNode {
    fn new_from_index(_: u32) -> Self {
        EmptyNode { }
    }

    fn make_string(&self) -> Option<String> {
        Some("".to_string())
    }


    fn parse_str(_to_parse: &str) -> Option<(&str, Self)>
        where Self: Sized
    {
        Some((_to_parse, EmptyNode{ }))
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
