//! I am writing this libary for my scientific simulations
//!
//! * you probably want to take a look at [`GenericGraph`](generic_graph/struct.GenericGraph.html).
//! * take a look at the module [`er_c`](er_c/index.html)
//!   or [`er_m`](er_m/index.html) if you want to do something with an Erdős-Rényi ensemble
//! * if you want to work with small-world ensemble, look at module [`sw`](sw/index.html)
//! * an example for implementing your own Node can be found [here](graph/type.Graph.html#example-2).
//!   Note that the defined Node can be used in the Graph ensembles
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
//! You can also compute different measurable quantities, look at
//! [`GenericGraph`](generic_graph/struct.GenericGraph.html) for more.
//!
//! ```
//! use net_ensembles::*;
//! use rand_pcg::Pcg64;
//! use rand::SeedableRng;
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
//! **Note:** Also works for small-world ensemble, i.e. for
//! [`SwEnsemble`](sw/struct.SwEnsemble.html)
//! # Example 3
//! ## Simple sample for small-world ensemble
//! * **Note:** simple sampling also works for [`ErEnsembleC`](er_c/struct.ErEnsembleC.html)
//! and [`ErEnsembleM`](er_m/struct.ErEnsembleM.html)
//! * see trait [```SimpleSample```](./traits/trait.SimpleSample.html)
//! ```
//! use net_ensembles::{SwEnsemble, EmptyNode};
//! use net_ensembles::traits::*; // I recommend always using this
//! use rand_pcg::Pcg64; //or whatever you want to use as rng
//! use rand::SeedableRng; // I use this to seed my rng, but you can use whatever
//! use std::fs::File;
//! use std::io::{BufWriter, Write};
//!
//! let rng = Pcg64::seed_from_u64(1822);
//!
//! // now create small-world ensemble with 100 nodes
//! // and a rewiring probability of 0.3 for each edge
//! let mut sw_ensemble = SwEnsemble::<EmptyNode, Pcg64>::new(100, 0.3, rng);
//!
//! // setup file for writing
//! let f = File::create("simple_sample_sw.dat")
//!     .expect("Unable to create file");
//! let mut f = BufWriter::new(f);
//! f.write_all(b"#diameter bi_connect_max average_degree\n")
//!     .unwrap();
//!
//! // simple sample for 10 steps
//! sw_ensemble.simple_sample(10,
//!     |ensemble|
//!     {
//!         let diameter = ensemble.graph()
//!             .diameter()
//!             .unwrap();
//!
//!         let bi_connect_max = ensemble.graph()
//!             .clone()
//!             .vertex_biconnected_components(false)[0];
//!
//!         let average_degree = ensemble.graph()
//!             .average_degree();
//!
//!         write!(f, "{} {} {}\n", diameter, bi_connect_max, average_degree)
//!             .unwrap();
//!     }
//! );
//!
//! // or just collect this into a vector to print or do whatever
//! let vec = sw_ensemble.simple_sample_vec(10,
//!     |ensemble|
//!     {
//!         let diameter = ensemble.graph()
//!             .diameter()
//!             .unwrap();
//!
//!         let transitivity = ensemble.graph()
//!             .transitivity();
//!         (diameter, transitivity)
//!     }
//! );
//! println!("{:?}", vec);
//! ```
//!
//! # Example 4: Save and load
//! * only works if feature ```"serde_support"``` is enabled
//! * Note: ```"serde_support"``` is enabled by default
//! * I need the ```#[cfg(feature = "serde_support")]``` to ensure the example does compile if
//!  you opt out of the default feature
//! * you can do not have to use ```serde_json```, look [here](https://docs.serde.rs/serde/) for more info
//! ```
//! use net_ensembles::traits::*; // I recommend always using this
//! use serde_json;
//! use rand_pcg::Pcg64;
//! use net_ensembles::{ErEnsembleC, EmptyNode, rand::SeedableRng};
//! use std::fs::File;
//!
//! let rng = Pcg64::seed_from_u64(95);
//! // create Erdős-Rényi ensemble
//! let ensemble = ErEnsembleC::<EmptyNode, Pcg64>::new(200, 3.1, rng);
//!
//! #[cfg(feature = "serde_support")]
//! {
//!     // storing the ensemble in a file:
//!
//!     let er_file = File::create("erC_save.dat")
//!           .expect("Unable to create file");
//!
//!     // or serde_json::to_writer(er_file, &ensemble);
//!     serde_json::to_writer_pretty(er_file, &ensemble);
//!
//!     // loading ensemble from file:
//!
//!     let mut read = File::open("erC_save.dat")
//!         .expect("Unable to open file");
//!
//!     let er: ErEnsembleC::<EmptyNode, Pcg64> = serde_json::from_reader(read).unwrap();
//! }
//! ```
//! # Example 5: Marcov Chain
//! * example for a Marcov chain of connected graphs
//! * you can also create a Marcov chain with unconnected graphs if you want
//! * see trait [```MarcovChain```](./traits/trait.MarkovChain.html)
//! ```
//! use net_ensembles::{EmptyNode, ErEnsembleM, traits::*};
//! use rand_pcg::Pcg64;
//! use net_ensembles::rand::SeedableRng; // rand is reexported
//!
//! // first create the ensemble
//! let rng = Pcg64::seed_from_u64(8745);
//! let mut e = ErEnsembleM::<EmptyNode, Pcg64>::new(30, 70, rng);
//!
//! // ensure initial graph is connected
//! while !e.graph()
//!     .is_connected().unwrap() {
//!     e.randomize();
//! }
//!
//! // Create marcov chain, e.g., of connected graphs
//! for _ in 0..100 {
//!     let steps = e.m_steps(10);
//!
//!     // reject, if the resulting graph is not connected
//!     if !e.graph().is_connected().unwrap() {
//!         e.undo_steps_quiet(steps);
//!     }
//!     // mesure whatever you want
//! }
//! ```
#![deny(missing_docs, warnings)]
pub mod generic_graph;
pub mod example_nodes;
pub mod graph;
pub mod er_c;
pub mod sw;
pub mod er_m;
mod graph_traits;
mod ensemble_traits;
pub mod traits;
pub mod dot_constants;
pub mod sw_graph;
pub mod iter;
pub mod step_structs;

pub use sw::SwEnsemble;
pub use sw_graph::SwGraph;
pub use er_m::ErEnsembleM;
pub use er_c::ErEnsembleC;
pub use graph::Graph;
pub use generic_graph::GenericGraph;
pub use example_nodes::EmptyNode;
pub use graph_traits::GraphErrors;
pub use traits::*;
pub use iter::IterWrapper;
pub use step_structs::*;

pub use rand;
