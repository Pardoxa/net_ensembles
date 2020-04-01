//! # constants for dot options
//!
//! You can chain options by joining them:
//! ```
//! use net_ensembles::dot_constants::*;
//! let chained = &[SPLINES, TRANSPARENT_BG, NO_OVERLAP]
//!        .join("\n\t");
//! ```

/// Example options. You are free to use your own. Search for graphviz.
pub const EXAMPLE_DOT_OPTIONS: &str = "bgcolor=\"transparent\";\n\tfontsize=50;\n\t\
        node [shape=ellipse, penwidth=1, \
        fontname=\"Courier\", pin=true ];\n\tsplines=true;";

/// * activate splines for `to_dot_with_labels_from_contained` or
/// `to_dot_with_labels_from_container`
///
/// **Note:** You can chain options by joining the strings
pub const SPLINES: &str = "splines=true;";

/// * use transparent background for `to_dot_with_labels_from_contained` or
/// `to_dot_with_labels_from_container`
///
/// **Note:** You can chain options by joining the strings
pub const TRANSPARENT_BG: &str = "bgcolor=\"transparent\";";

/// * no overlapping nodes for `to_dot_with_labels_from_contained` or
/// `to_dot_with_labels_from_container`
///
/// **Note:** You can chain options by joining the strings
pub const NO_OVERLAP: &str = "overlap=false;";
