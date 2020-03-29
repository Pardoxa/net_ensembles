//! I am writing this libary for my scientific simulations
//!
//! * You probably want to take a look at the struct `Graph` from the module `graph`.
//! * Also, take a look at the module `er` if you want to do something with Erdős-Rényi networks
//! # Example 1
//! Create an Erdős-Rényi graph
//! ```
//! use net_ensembles::*;
//! use rand_pcg::Pcg64;
//! use rand::SeedableRng;
//! use std::fs::File;
//! use std::io::prelude::*;
//!
//! let rng = Pcg64::seed_from_u64(75676526);
//! // create graph with 50 vertices and target connectivity of 2.7
//! // using Pcg64 as random number generator
//! let mut er_graph = ER::<EmptyNode, Pcg64>::new(50, 2.7, rng);
//! // create dot file to visualize the graph
//! let dot = er_graph.graph().to_dot();
//! let mut f = File::create("50.dot")
//!                    .expect("Unable to create file");
//! f.write_all(dot.as_bytes())
//!     .expect("Unable to write data");
//!
//! er_graph.randomize();
//! let dot = er_graph.graph().to_dot();
//! let mut f = File::create("50_1.dot")
//!                    .expect("Unable to create file");
//! f.write_all(dot.as_bytes())
//!     .expect("Unable to write data");
//!
//! ```
//! To visualize, you can use something like
//! ```dot
//! twopi 50.dot -Tpdf > 50.pdf
//! circo 50_1.dot -Tpdf > 50_1.pdf
//! ```
//! You can also try some of the other [roadmaps](https://www.graphviz.org/).
//! # Example 2
//! You can also compute different measurable quantities, look at `graph::Graph` for more.
//! ```
//! use net_ensembles::*;
//! use rand_pcg::Pcg64;
//! use rand::SeedableRng;
//! use std::fs::File;
//! use std::io::prelude::*;
//!
//! let rng = Pcg64::seed_from_u64(26);
//! // create graph with 50 vertices and target connectivity of 2.7
//! // using Pcg64 as random number generator
//! let er = ER::<EmptyNode, Pcg64>::new(50, 2.7, rng);
//! println!("Number of vertices: {}",      er.graph().vertex_count());
//! println!("Number of edges: {}",         er.graph().edge_count());
//! println!("Average degree: {}",          er.graph().average_degree());
//! println!("connected components: {:?}",  er.graph().connected_components());
//! println!("transitivity: {}",            er.graph().transitivity());
//! ```
pub mod node;
pub mod graph;
pub mod er;

pub use node::Node;
pub use node::TestNode;
pub use node::EmptyNode;
pub use er::ER;
pub use er::ErStep;
pub use graph::Graph;
pub use graph::DEFAULT_DOT_OPTIONS;
