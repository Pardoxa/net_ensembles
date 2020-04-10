//! The structs returned by the mc steps
//!
//! if you want to use the `Ensemble` trait
//! as trait bound, use e.g. `Ensemble<ErStepC, ErStepC>`
//! for trait implmented in `ErEnsembleC`


pub use crate::sw::SwChangeState;
pub use crate::er_c::ErStepC;
pub use crate::er_m::ErStepM;
/// retunred by mc step of small world ensemble
pub type SwStep = SwChangeState;
