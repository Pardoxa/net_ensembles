use std::borrow::*;
use std::io::Write;
use std::default::Default;

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
/// # Settings for gnuplot
/// * implements default
/// * implements builder pattern for itself
pub struct GnuplotSettings{
    /// x label for gnuplog
    pub x_label: String,
    /// y label for gnuplot
    pub y_label: String,
    /// title for gnuplot
    pub title: String,
    /// which terminal to use for gnuplot
    pub terminal: GnuplotTerminal,
}

impl GnuplotSettings {
    /// # Builder pattern - set x_label
    pub fn x_label<'a>(&'a mut self, x_label: String) -> &'a mut Self
    {
        self.x_label = x_label;
        self
    }

    /// # Builder pattern - set y_label
    pub fn y_label<'a>(&'a mut self, y_label: String) -> &'a mut Self
    {
        self.y_label = y_label;
        self
    }

    /// # Builder pattern - set title
    pub fn title<'a>(&'a mut self, title: String) -> &'a mut Self
    {
        self.title = title;
        self
    }

    /// # Builder pattern - set terminal
    pub fn terminal<'a>(&'a mut self, terminal: GnuplotTerminal) -> &'a mut Self
    {
        self.terminal = terminal;
        self
    }
}

impl Default for GnuplotSettings{
    fn default() -> Self {
        Self{
            x_label: "".to_owned(),
            y_label: "".to_owned(),
            title: "".to_owned(),
            terminal: GnuplotTerminal::PDF,
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