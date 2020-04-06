//! I am writing this libary for my scientific simulations
//!
//! * you probably want to [take a look](generic_graph/struct.GenericGraph.html) at `net_ensembles::GenericGraph`.
//! * [take a look](er/index.html) at the module `er` if you want to do something with an Erdős-Rényi ensemble
//! * if you want to work with small-world ensemble, [look at](sw/index.html) module `sw`
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
//! let mut er_graph = ErEnsembleC::<EmptyNode, Pcg64>::new(50, 2.7, rng);
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
//! let er = ErEnsembleC::<EmptyNode, Pcg64>::new(50, 2.7, rng);
//! println!("Number of vertices: {}",      er.graph().vertex_count());
//! println!("Number of edges: {}",         er.graph().edge_count());
//! println!("Average degree: {}",          er.graph().average_degree());
//! println!("connected components: {:?}",  er.graph().connected_components());
//! println!("transitivity: {}",            er.graph().transitivity());
//! ```
pub mod generic_graph;
pub mod example_nodes;
pub mod graph;
pub mod er_c;
pub mod sw;
mod graph_traits;
pub mod traits;
pub mod dot_constants;
pub mod sw_graph;
mod iter;

pub use sw::SwEnsemble;
pub use sw_graph::SwGraph;
pub use er_c::ErEnsembleC;
pub use er_c::ErStepC;
pub use graph::Graph;
pub use generic_graph::GenericGraph;
pub use example_nodes::TestNode;
pub use example_nodes::EmptyNode;
pub use graph_traits::GraphErrors;
pub use graph_traits::SwChangeState;
pub use traits::*;
pub use iter::IterWrapper;
