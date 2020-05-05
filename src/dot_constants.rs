//! # constants for dot options
//!
//! ## Example
//! ```
//! use net_ensembles::{*, rand::SeedableRng, dot_constants::*};
//! use rand_pcg::Pcg64;
//!
//! let rng = Pcg64::seed_from_u64(0);
//! let ensemble = ErEnsembleM::<EmptyNode, _>::new(10, 20, rng);
//!
//! // create dot
//! let dot = ensemble.graph().to_dot_with_labels_from_contained(
//!     dot_options!(TRANSPARENT_BG, NO_OVERLAP, SIZE_A4, RATIO_FILL),
//!     |_, index|
//!     {
//!         format!("Hey, I am at index: {}", index)
//!     }
//! );
//!
//! println!("{}", dot);
//! ```


/// You can chain/combine options with the `dot_options!` macro:
/// ```
/// use net_ensembles::dot_constants::*;
/// use net_ensembles::dot_options;
///
/// dot_options!(SPLINES, TRANSPARENT_BG, NO_OVERLAP, "fontsize=50;");
///
/// // Note, the macro is equivalent to
/// let chain = [SPLINES, TRANSPARENT_BG].join("\n\t");
///
/// assert!(chain == dot_options!(SPLINES, TRANSPARENT_BG) );
/// ```
#[macro_export]
macro_rules! dot_options {
    ( $( $x:expr ),* ) => {
        [ $( $x ),* ].join("\n\t")
    }
}

/// * Example options. You are free to use your own. Search for graphviz.
pub const EXAMPLE_DOT_OPTIONS: &str = "bgcolor=\"transparent\";\n\tfontsize=50;\n\t\
        node [shape=ellipse, penwidth=1, \
        fontname=\"Courier\", pin=true ];\n\tsplines=true;";

/// * activate splines for for `to_dot*`
///
/// **Note:** You can chain options by joining the strings
pub const SPLINES: &str = "splines=true;";

/// * use transparent background for `to_dot*`
///
/// **Note:** You can chain options by joining the strings
pub const TRANSPARENT_BG: &str = "bgcolor=\"transparent\";";

/// * no overlapping nodes for `to_dot*`
///
/// **Note:** You can chain options by joining the strings
pub const NO_OVERLAP: &str = "overlap=false;";

/// * Din A4 size
pub const SIZE_A4: &str = "size=\"8.3,11.7!\";";

/// * dot option: ratio="fill"
pub const RATIO_FILL: &str = "ratio=\"fill\";";

/// * do not use margin
pub const MARGIN_0: &str = "margin=0;";
