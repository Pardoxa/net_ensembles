/// What every node should be able to do
///
pub trait Node {
    fn new_empty() -> Self;
}


struct TestNode {
    info: String,
}

impl TestNode {
    fn get_info(&self) -> &str {
        &self.info
    }
}

impl Node for TestNode {
    fn new_empty() -> Self {
        TestNode {
            info: "".to_string()
        }
    }
}

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn t1() {
        let n = TestNode::new_empty();
        assert_eq!("", n.get_info());
    }
}
