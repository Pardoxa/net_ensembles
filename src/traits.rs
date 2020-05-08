//! # You should `use net_ensembles::traits::*`
//! * I recommend doing so for a smooth experience
//! * contains traits you should use for accessing complete functionallity
pub use crate::graph_traits::Node;
pub use crate::graph_traits::AdjContainer;
pub use crate::graph_traits::MeasurableGraphQuantities;
pub use crate::ensemble_traits::MarkovChain;
pub use crate::ensemble_traits::SimpleSample;
pub use crate::ensemble_traits::HasRng;
pub use crate::ensemble_traits::WithGraph;
pub use crate::ensemble_traits::GraphIterators;
pub use crate::ensemble_traits::GraphIteratorsMut;
pub use crate::iter::IterWrapper;

mod dot_traits;
pub use dot_traits::{Dot, DotExtra};

#[cfg(feature = "serde_support")]
use serde::{Serialize};

/// * intermediate trait used for trait bounds
/// * if feature "serde_support" is used, there is a blanked implementation for all
/// types that implement ```serde::Serialize```
/// * else there is a blanked implementation for all types
#[cfg(feature = "serde_support")]
pub trait SerdeStateConform : Serialize {}

/// * intermediate trait used for trait bounds
/// * if feature "serde_support" is used, there is a blanked implementation for all
/// types that implement ```serde::Serialize```
/// * else there is a blanked implementation for all types
#[cfg(not(feature = "serde_support"))]
pub trait SerdeStateConform {}

#[cfg(feature = "serde_support")]
impl<T> SerdeStateConform for T
where T: Serialize {}

#[cfg(not(feature = "serde_support"))]
impl<T> SerdeStateConform for T {}
