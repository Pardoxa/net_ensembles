//! # This lib is intended for scientific simulations
//!
//! * you probably want to take a look at [`GenericGraph`](generic_graph/struct.GenericGraph.html).
//! * take a look at the module [`er_c`](er_c/index.html)
//!   or [`er_m`](er_m/index.html) if you want to do something with an **Erdős-Rényi** ensemble
//! * if you want to work with a **small-world** ensemble, look at module [`sw`](sw/index.html)
//! * an example for implementing your own Node can be found [here](graph/type.Graph.html#example-2).
//!   Note that the defined Node can be used in the Graph ensembles
//! * Note: The ensembles implement the trait [`GraphIterators`](./traits/trait.GraphIterators.html),
//! therefore calling, e.g., `ensemble.graph().dfs(0)` is equivalent
//! to `ensemble.dfs(0)` as long as you used
//! `use::net_ensembles::traits::*`
//! * See also: [`GraphIteratorsMut`](./traits/trait.GraphIteratorsMut.html)
//! * Note: the ensembles implement the trait
//! [`MeasurableGraphQuantities`](./traits/trait.MeasurableGraphQuantities.html),
//! therefore, e.g., `ensemble.graph().transitivity()` and `ensemble.transitivity()`
//! are equivalent
//! * for **sampling** the ensemble, take a look at [these traits](./sampling/traits/index.html)
//! # Example 1
//!
//! ```
//! use net_ensembles::{ErEnsembleC, EmptyNode, rand::SeedableRng};
//! use net_ensembles::traits::{WithGraph, SimpleSample, Dot};
//! use net_ensembles::{dot_options, dot_constants::*};
//! // Note: you might have to enable serde for rand_pcg
//! // to do that, write the following in your Cargo.toml:
//! // rand_pcg = { version = "*", features = ["serde1"]}
//! use rand_pcg::Pcg64;
//! use std::fs::File;
//!
//! let rng = Pcg64::seed_from_u64(75676526);
//! // create graph with 50 vertices and target connectivity of 2.7
//! // using Pcg64 as random number generator
//! // NOTE: you can exchange `EmptyNode` with anything implementing the `Node` trait
//! let mut er = ErEnsembleC::<EmptyNode, _>::new(50, 2.7, rng);
//!
//! // create dot file to visualize the graph
//! let mut f = File::create("50.dot")
//!                    .expect("Unable to create file");
//! // look at Dot trait
//! er.graph().dot(
//!      f,
//!      "" // you do not have to use dot_options
//!  ).unwrap();
//!
//! // randomize the graph, uses SimpleSample trait
//! er.randomize();
//!
//! let mut f = File::create("50_1.dot")
//!                    .expect("Unable to create file");
//! er.graph().dot_with_indices(
//!     f,
//!     dot_options!(NO_OVERLAP, MARGIN_0)
//! ).unwrap();
//!
//! // Note, you can also create a String this way:
//! let s =  er.graph().dot_string("");
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
//! use net_ensembles::{traits::*, EmptyNode, ErEnsembleC};
//! use rand_pcg::Pcg64;
//! use rand::SeedableRng;
//!
//! let rng = Pcg64::seed_from_u64(26);
//! // create graph with 50 vertices and target connectivity of 2.7
//! // using Pcg64 as random number generator
//! let er = ErEnsembleC::<EmptyNode, Pcg64>::new(50, 2.7, rng);
//! println!("Number of vertices: {}",      er.vertex_count());
//! println!("Number of edges: {}",         er.edge_count());
//! println!("Average degree: {}",          er.average_degree());
//! println!("connected components: {:?}",  er.connected_components());
//! println!("transitivity: {}",            er.transitivity());
//! ```
//! **Note:** Also works for small-world ensemble, i.e. for
//! [`SwEnsemble`](sw/struct.SwEnsemble.html)
//! # Example 3
//! ## Simple sample for small-world ensemble
//! * **Note:** simple sampling also works for [`ErEnsembleC`](er_c/struct.ErEnsembleC.html)
//! and [`ErEnsembleM`](er_m/struct.ErEnsembleM.html)
//! * see trait [```SimpleSample```](./sampling/traits/trait.SimpleSample.html)
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
//!         let diameter = ensemble
//!             .diameter()
//!             .unwrap();
//!
//!         let bi_connect_max = ensemble.graph().clone()
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
//! * you do not have to use ```serde_json```, look [here](https://docs.serde.rs/serde/) for more info
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
//! * see trait [```MarcovChain```](./sampling/traits/trait.MarkovChain.html)
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
//!
//! # Example 6: Define your own Data
//! * Note: You will not need the cfg parts, though you have to
//!  use ```#[derive(Serialize, Deserialize)]``` if you use the
//!  default features of ```net_ensembles```
//! ```
//! use net_ensembles::{traits::*, rand::SeedableRng, SwEnsemble};
//! use rand_pcg::Pcg64;
//!
//! #[cfg(feature = "serde_support")]
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Clone, PartialEq)]
//! #[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
//! pub enum SirState{
//!     Susceptible,
//!     Infective,
//!     Removed,
//! }
//!
//! impl SirState {
//!     fn setState(&mut self, state: Self) {
//!         *self = state;
//!     }
//! }
//!
//! impl Node for SirState {
//!     fn new_from_index(index: usize) -> Self {
//!         SirState::Susceptible
//!     }
//! }
//!
//! // create the rng:
//! let rng = Pcg64::seed_from_u64(45);
//!
//! let mut ensemble = SwEnsemble::<SirState, Pcg64>::new(10, 0.1, rng);
//!
//! // you can access or change your additional information, e.g., at vertex 0
//! ensemble
//!     .at_mut(0)
//!     .setState(SirState::Infective);
//!
//! // you can also iterate over your additional information:
//! let count = ensemble
//!     .contained_iter()
//!     .filter(|&state| *state == SirState::Susceptible)
//!     .count();
//! assert!(count == 9);
//!
//! // or count how many Susceptible nodes are connected to a specific node, i.e., to node 0
//! let s_count = ensemble
//!     .contained_iter_neighbors(0)
//!     .filter(|&state| *state == SirState::Susceptible)
//!     .count();
//!
//! println!("{}", s_count);
//!
//! // or advance the states:
//! for state in ensemble.contained_iter_mut() {
//!     match *state {
//!         SirState::Infective     => { *state = SirState::Removed },
//!         _                       => { },
//!     };
//! }
//! ```
#![deny(missing_docs, warnings)]
pub mod generic_graph;
pub mod example_nodes;
pub mod graph;
pub mod er_c;
pub mod sw;
pub mod er_m;
mod barabasi_albert;
pub mod traits;
#[macro_use]
pub mod dot_constants;
pub mod sw_graph;
pub mod iter;
pub mod step_structs;
pub mod sampling;

pub use sw::SwEnsemble;
pub use sw_graph::SwGraph;
pub use er_m::ErEnsembleM;
pub use er_c::ErEnsembleC;
pub use barabasi_albert::BAensemble;
pub use graph::Graph;
pub use generic_graph::GenericGraph;
pub use example_nodes::EmptyNode;
pub use traits::*;
pub use traits::GraphErrors;
pub use iter::IterWrapper;
pub use step_structs::*;

pub use rand;
