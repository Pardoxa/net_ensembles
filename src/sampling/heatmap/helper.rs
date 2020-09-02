use crate::sampling::*;
use std::borrow::*;
use std::io::Write;

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

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
/// # Options for choosing gnuplot Terminal
pub enum GnuplotTerminal{
    /// # Use EpsLatex as terminal in gnuplot
    /// * the created gnuplotscript assumes, you have `latexmk` installed
    /// * if you do not have latexmk, you can still use this, but you have to manually edit the 
    /// gnuplotskrip later on
    /// * gnuplot skript will create `.tex` file and `.pdf` file created from the tex file
    EpsLatex,
    /// # Use pdf as gnuplot terminal
    /// * gnuplot skript will create a `.pdf` file
    PDF,
}

impl GnuplotTerminal{
    pub(crate) fn terminal(&self) -> &'static str
    {
        match self{
            Self::EpsLatex => {
                "set t epslatex 9 standalone color size 7.4cm, 5cm header \"\\\\usepackage{amsmath}\\n\"\nset font \",9\""
            },
            Self::PDF => {
                "set t pdf"
            }
        }
    }

    pub(crate) fn output(&self, name: &str) -> String
    {
        let mut name = name.to_owned();
        match self {
            Self::EpsLatex => {
                if name.ends_with(".tex") {
                    name
                } else {
                    name.push_str(".tex");
                    name
                }
            },
            Self::PDF => {
                if name.ends_with(".pdf") {
                    name
                } else {
                    name.push_str(".pdf");
                    name
                }
            }
        }
    }

    pub(crate) fn finish<W: Write>(&self, output_name: &str, mut w: W) -> std::io::Result<()>
    {
        match self {
            Self::EpsLatex => writeln!(w, "system('latexmk {} -pdf -f')", output_name),
            _ => Ok(())
        }
    } 
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
