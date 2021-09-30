//! # For sampling ensembles
//! * contains Simple sampling, WangLandau, entropic sampling, Metropolis, Histograms
//! * This is just for compatibility reasons, everything was moved into its 
//! own crate, which you can find at [crates.io/crates/sampling](https://crates.io/crates/sampling)

/// Contains traits useful for sampling an ensemble
/// like MarkovChain or Metropolis etc.
pub use sampling::*;