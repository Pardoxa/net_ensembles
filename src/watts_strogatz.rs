//! # Watts-Strogatz small-world networks
//! This module implements a small-world ensemble.
//! 
//! What is the difference to [SwEnsemble](`crate::sw`) you ask?
//! Here the edges are not attatched to a root edge, which means,
//! that it is possible to generate leafs with this ensemble.
//! It is even possible, that nodes have a degree of 0,
//! because all their edges could be rewired elsewhere
//!
//! # Citations
//! > D. J. Watts and S. H. Strogatz, "Collective dynamics on 'small-world' networks,"
//!   Nature **393**, 440-442 (1998), DOI:&nbsp;[10.1038/30918](https://doi.org/10.1038/30918)
mod ws_container;
mod ensemble;

pub use ensemble::*;
pub use ws_container::*;