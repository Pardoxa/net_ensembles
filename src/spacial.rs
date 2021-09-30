//! # Spacial ensemble
//!
//! # Citation
//! 
//! Overview over spacial networks:
//! > Marc Barthélemy,
//! > "Spacial networks" *Physics Reports*&nbsp;**499**:1-101&nbsp;(2011),
//! > DOI: [10.1016/j.physrep.2010.11.002](https://doi.org/10.1016/j.physrep.2010.11.002)
//! 
//! The specific model I implemented is described in
//! > Timo Dewenter and Alexander K. Hartmann,
//! > "Large-deviation properties of resilience of power grids"
//! > *New&nbsp;J.&nbsp;Phys.*&nbsp;**17**&nbsp;(2015),
//! > DOI: [10.1088/1367-2630/17/1/015005](https://doi.org/10.1088/1367-2630/17/1/015005)
mod spacial_graph;
mod spacial_ensemble;

pub use spacial_graph::*;
pub use spacial_ensemble::*;