use std::borrow::*;
use std::io::Write;
use std::default::Default;

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
/// For labeling the gnuplot plots axis
pub enum GnuplotAxis{
    /// construct the labels
    FromValues{
        /// minimum value for axis labels
        min: f64,
        /// maximum value for axis labels
        max: f64,
        ///number of tics, should be at least 2
        tics: usize,
    },
    /// use labels 
    Labels{
        /// this are the labels
        labels: Vec<String>
    }
}

impl GnuplotAxis{
    pub(crate) fn write_tics<W: Write>(&self, mut w: W, num_bins: usize, axis: &str) -> std::io::Result<()>
    {
        match self {
            Self::FromValues{min, max, tics} => {
                if min.is_nan() || max.is_nan() || *tics < 2 || num_bins < 2 {
                    Ok(())
                } else {
                    let t_m1 = tics - 1;
                    let difference = (max - min) / t_m1 as f64;
                    dbg!(difference);
                    let bin_dif = (num_bins - 1) as f64 / t_m1 as f64;
                    write!(w, "set {}tics ( ", axis)?;
                    for i in  0..t_m1 {
                        dbg!(i as f64 * difference);
                        let val = min + i as f64 * difference;
                        let pos = i as f64 * bin_dif;
                        write!(w, "\"{}\" {}, ", val, pos)?; 
                    }
                    writeln!(w, "\"{}\" {} )", max,  num_bins - 1)
                }
            }, 
            Self::Labels{labels} => {
                let tics = labels.len();
                match tics {
                    0 => Ok(()),
                    1 => {
                        writeln!(w, "set {}tics ( \"{}\" 0 )", axis, labels[0])
                    },
                    _ => {
                        write!(w, "set {}tics ( ", axis)?;
                        let t_m1 = tics - 1;
                        let bin_dif = (num_bins - 1) as f64 / t_m1 as f64;
                        for (i, lab) in labels.iter().enumerate(){
                            let pos = i as f64 * bin_dif;
                            write!(w, "\"{}\" {}, ", lab, pos)?; 
                        }
                        writeln!(w, " )")
                    }
                }
            }
        }
        
    }

    pub fn new(min: f64, max: f64, tics: usize) -> Self {
        Self::FromValues{
            min,
            max,
            tics
        }
    }

    pub fn from_labels(labels: Vec<String>) -> Self
    {
        Self::Labels{
            labels
        }
    }

    pub fn from_slice(labels: &[&str]) -> Self {
        let vec = labels.iter()
            .map(|&s| s.into())
            .collect();
        
        Self::Labels{
            labels: vec
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
/// # Settings for gnuplot
/// * implements default
/// * implements builder pattern for itself
pub struct GnuplotSettings{
    /// x label for gnuplog
    pub x_label: String,
    /// how to format the lables of the x axis?
    pub x_axis: Option<GnuplotAxis>,
    /// y label for gnuplot
    pub y_label: String,
    /// how to format the lables of the y axis?
    pub y_axis: Option<GnuplotAxis>,
    /// title for gnuplot
    pub title: String,
    /// which terminal to use for gnuplot
    pub terminal: GnuplotTerminal,

    /// Color pallet for heatmap
    pub pallet: GnuplotPallet,
}

impl GnuplotSettings {
    /// # Builder pattern - set x_label
    pub fn x_label<'a, S: Into<String>>(&'a mut self, x_label: S) -> &'a mut Self
    {
        self.x_label = x_label.into();
        self
    }

    pub(crate) fn write_label<W: Write>(&self, mut writer: W) -> std::io::Result<()>
    {
        if !self.x_label.is_empty(){
            writeln!(writer, "set xlabel \"{}\"", self.x_label)?;
        }
        if !self.y_label.is_empty(){
            writeln!(writer, "set ylabel \"{}\"", self.y_label)
        } else {
            Ok(())
        }
    }

    /// # Builder pattern - set y_label
    pub fn y_label<'a, S: Into<String>>(&'a mut self, y_label: S) -> &'a mut Self
    {
        self.y_label = y_label.into();
        self
    }

    /// # Builder pattern - set title
    pub fn title<'a, S: Into<String>>(&'a mut self, title: S) -> &'a mut Self
    {
        self.title = title.into();
        self
    }

    /// # currently set title
    pub fn get_title(&self) -> &str
    {
        &self.title
    }

    /// # Builder pattern - set terminal
    pub fn terminal<'a>(&'a mut self, terminal: GnuplotTerminal) -> &'a mut Self
    {
        self.terminal = terminal;
        self
    }

    pub(crate) fn terminal_str(&self) -> &'static str {
        self.terminal.terminal_str()
    }

    pub fn pallet<'a>(&'a mut self, pallet: GnuplotPallet) -> &'a mut Self
    {
        self.pallet = pallet;
        self
    }

    /// Create new, default, GnuplotSettings
    pub fn new() -> Self
    {
        Self::default()
    }

    pub fn x_axis<'a>(&'a mut self, axis: GnuplotAxis) -> &'a mut Self
    {
        self.x_axis = Some(axis);
        self
    }

    pub fn y_axis<'a>(&'a mut self, axis: GnuplotAxis) -> &'a mut Self
    {
        self.y_axis = Some(axis);
        self
    }

    pub(crate) fn write_axis<W: Write>(&self, mut w: W, num_bins_x: usize, num_bins_y: usize) -> std::io::Result<()>
    {
        if let Some(ax) = self.x_axis.as_ref() {
            ax.write_tics(&mut w, num_bins_x, "x")?;
        }
        if let Some(ax) = self.y_axis.as_ref() {
            ax.write_tics(w, num_bins_y, "y")?;
        }
        Ok(())
    }
}

impl Default for GnuplotSettings{
    fn default() -> Self {
        Self{
            x_label: "".to_owned(),
            y_label: "".to_owned(),
            title: "".to_owned(),
            terminal: GnuplotTerminal::PDF,
            pallet: GnuplotPallet::PresetHSV,
            x_axis: None,
            y_axis: None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
/// defines presets for different color pallets
pub enum GnuplotPallet{
    PresetHSV,
    PresetRGB
}

impl GnuplotPallet{
    pub(crate) fn write_pallet<W: Write>(&self, mut writer: W) -> std::io::Result<()>
    {
        match self {
            Self::PresetHSV => {
                writeln!(writer, "set palette model HSV")?;
                writeln!(writer, "set palette negative defined  ( 0 0 1 0, 2.8 0.4 0.6 0.8, 5.5 0.83 0 1 )")
            },
            Self::PresetRGB => Ok(())
        }
        
    }
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
    pub(crate) fn terminal_str(&self) -> &'static str
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
    
    pub(crate) fn output<W: Write>(&self, name: &str, mut writer: W) -> std::io::Result<()>
    {
        match self {
            Self::EpsLatex => {
                if name.ends_with(".tex") {
                    write!(writer, "{}", name)
                } else {
                    write!(writer, "{}.tex", name)
                }
            },
            Self::PDF => {
                if name.ends_with(".pdf") {
                    write!(writer, "{}", name)
                } else {
                    write!(writer, "{}.pdf", name)
                }
            }
        }
    }

    pub(crate) fn finish<W: Write>(&self, output_name: &str, mut w: W) -> std::io::Result<()>
    {
        match self {
            Self::EpsLatex => {
                write!(w, "system('latexmk ")?;
                self.output(output_name, &mut w)?;
                writeln!(w, "-pdf -f')")
            },
            _ => Ok(())
        }
    } 
}