//! # For sampling ensembl
//! contains traits and helper structs

/// Contains traits useful for sampling an ensemble
/// like MarkovChain or Metropolis etc.
pub mod traits;
mod metropolis_helper;
mod wang_landau;

pub use wang_landau::*;


pub use metropolis_helper::*;
