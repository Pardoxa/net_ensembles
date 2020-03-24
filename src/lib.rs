//! Libary for my simulations
//! Here I will define the node structs etc.
//!
//! * You probably want to take a look at the struct `Graph` from the module `graph`.
//! * Also, take a look at the module `er` if you want to do something with Erdős-Rényi networks
//! # Example
//! ```
//! use net_ensembles;
//! use net_ensembles::Node;
//!
//! ```
pub mod node;
pub mod graph;
pub mod er;
pub use node::Node;
pub use er::ER;
pub use graph::Graph;
pub use node::TestNode;
pub use graph::DEFAULT_DOT_OPTIONS;
