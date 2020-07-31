//! # For sampling ensembles
//! * contains Simple sampling, WangLandau, Metropolis, Histograms

/// Contains traits useful for sampling an ensemble
/// like MarkovChain or Metropolis etc.
pub mod traits;
mod metropolis_helper;
mod wang_landau;
mod histogram;

pub use wang_landau::*;


pub use metropolis_helper::*;
pub use histogram::*;