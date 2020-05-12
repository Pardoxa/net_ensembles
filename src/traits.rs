//! # You should `use net_ensembles::traits::*`
//! * I recommend doing so for a smooth experience
//! * contains traits you should use for accessing complete functionallity

mod graph_traits;
pub use graph_traits::Node;
pub use graph_traits::AdjContainer;
pub use graph_traits::MeasurableGraphQuantities;
pub use graph_traits::GraphErrors;

mod ensemble_traits;
pub use ensemble_traits::HasRng;
pub use ensemble_traits::WithGraph;
pub use ensemble_traits::GraphIterators;
pub use ensemble_traits::GraphIteratorsMut;

pub use crate::sampling::traits::*;
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
