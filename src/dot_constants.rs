/// # constant for dot options
/// ```
/// pub const EXAMPLE_DOT_OPTIONS: &str = "bgcolor=\"transparent\";\n\tfontsize=50;\n\t\
///         node [shape=ellipse, penwidth=1, \
///         fontname=\"Courier\", pin=true ];\n\tsplines=true;";
/// ```
pub const EXAMPLE_DOT_OPTIONS: &str = "bgcolor=\"transparent\";\n\tfontsize=50;\n\t\
        node [shape=ellipse, penwidth=1, \
        fontname=\"Courier\", pin=true ];\n\tsplines=true;";

/// * activate splines for `to_dot_with_labels_from_contained` or
/// `to_dot_with_labels_from_container`
///
/// **Note:** You can chain options by joining the strings
pub const DOT_SPLINES: &str = "splines=true;";

/// * use transparent background for `to_dot_with_labels_from_contained` or
/// `to_dot_with_labels_from_container`
///
/// **Note:** You can chain options by joining the strings
pub const DOT_TRANSPARENT_BG: &str = "bgcolor=\"transparent\";";

/// * no overlapping nodes for `to_dot_with_labels_from_contained` or
/// `to_dot_with_labels_from_container`
///
/// **Note:** You can chain options by joining the strings
pub const DOT_NO_OVERLAP: &str = "overlap=false;";
