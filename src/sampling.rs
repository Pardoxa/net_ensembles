//! # For sampling ensembles
//! * contains Simple sampling, WangLandau, entropic sampling, Metropolis, Histograms

/// Contains traits useful for sampling an ensemble
/// like MarkovChain or Metropolis etc.
pub mod traits;
mod metropolis_helper;
mod wang_landau;
mod histogram;
mod entropic_sampling;

pub use wang_landau::*;
pub use entropic_sampling::*;

pub use metropolis_helper::*;
pub use histogram::*;