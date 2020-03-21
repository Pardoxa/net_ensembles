//! Libary for my simulations
//! Here I will define the node structs etc.
//!
//! # Example
//! ```
//! use net_ensembles;
//!
//!
//! ```
mod node;
pub use self::node::Node;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
