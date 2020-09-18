use crate::sampling::*;

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
/// # Errors of Heatmap
pub enum HeatmapError{
    /// An Error while calculating the index of the x coordinate
    XError(HistErrors),
    /// An Error while calculating the index of the y coordinate
    YError(HistErrors),
    /// you tried to combine heatmaps of different Dimensions
    Dimension
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
/// Options. Choose, how the heatmap shall be normalized
pub enum HeatmapNormalization{
    /// Use [Heatmap::heatmap_normalized](struct.Heatmap.html#method.heatmap_normalized) for normalization
    NormalizeTotal,
    /// Use [Heatmap::heatmap_normalize_columns](struct.Heatmap.html#method.heatmap_normalize_columns) for normalization
    NormalizeColumn,
    /// Use [Heatmap::heatmap_normalize_rows](struct.Heatmap.html#method.heatmap_normalize_rows) for normalization
    NormalizeRow,
    /// heatmap as is, without normalizing or anything
    AsIs
}
