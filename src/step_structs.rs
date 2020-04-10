//! The structs returned by the mc steps
//!
//! if you want to use the `MarkovChain` trait
//! as trait bound, use e.g. `MarkovChain<ErStepC, ErStepC>`
//! for trait implmented in `ErEnsembleC`


pub use crate::sw::SwChangeState;
pub use crate::er_c::ErStepC;
pub use crate::er_m::ErStepM;
/// returned by mc step of small world ensemble
pub type SwStep = SwChangeState;
