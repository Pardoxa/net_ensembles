use crate::sampling::*;
use std::convert::*;
use std::path::*;
use std::fs::*;
use std::io::{BufWriter, Write};

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub enum HeatmapError{
    XError(HistErrors),
    YError(HistErrors)
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub enum GnuplotTerminal{
    EpsLatex,
    PDF,
}

impl GnuplotTerminal{
    fn terminal(&self) -> &'static str
    {
        match self{
            Self::EpsLatex => {
                "set t epslatex 9 standalone color size 7.4cm, 5cm header \"\\usepackage{amsmath}\\n\"\nset font \",9\""
            },
            Self::PDF => {
                "set t pdf"
            }
        }
    }

    fn output(&self, name: &str) -> String
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

    fn finish<W: Write>(&self, output_name: &str, mut w: W) -> std::io::Result<()>
    {
        match self {
            Self::EpsLatex => writeln!(w, "system('{} -pdf -f')", output_name),
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct Heatmap<HistX, HistY>{
    hist_x: HistX,
    hist_y: HistY,
    bins_x: usize,
    bins_y: usize,
    heatmap: Vec<usize>, // stored bins_x, bins_y
    error_count: usize
}

impl <HistX, HistY> Heatmap<HistX, HistY>
{
    /// x = j
    /// y = i
    #[inline(always)]
    fn index(&self, x: usize, y: usize) -> usize
    {
        y * self.bins_x + x
    }

    pub fn get(&self, x: usize, y: usize) -> Option<usize>
    {
        self.heatmap.get(self.index(x, y)).copied()
    }

    pub unsafe fn get_unchecked(&self, x: usize, y: usize) -> usize
    {
        *self.heatmap.get_unchecked(self.index(x, y))
    }

    pub fn bins_x(&self) -> usize
    {
        self.bins_x
    }

    pub fn bins_y(&self) -> usize
    {
        self.bins_y
    } 


    /// # Returns reference to current X Histogram
    /// * all `counts` are counted here -> this is a projection of the heatmap
    pub fn x_projection(&self) -> &HistX{
        &self.hist_x
    }

    /// # Returns reference to current Y Histogram
    /// * all `counts` are counted here -> this is a projection of the heatmap
    pub fn y_projection(&self) -> &HistY{
        &self.hist_y
    }
}


impl<HistX, HistY> Heatmap<HistX, HistY>
where 
    HistX: Histogram + std::fmt::Debug,
    HistY: Histogram + std::fmt::Debug,
{

    pub fn new(mut histogram_x: HistX, mut histogram_y: HistY) -> Self {
        let bins_x = histogram_x.bin_count();
        let bins_y = histogram_y.bin_count();
        histogram_x.reset();
        histogram_y.reset();
        let heatmap = vec![0; bins_x * bins_y];
        Self{
            bins_x,
            bins_y,
            heatmap,
            hist_x: histogram_x,
            hist_y: histogram_y,
            error_count: 0
        }
    }

    /// # counts how many bins were hit in total
    /// * Note: it calculates this in O(min(self.bins_x, self.bins_y))
    pub fn total(&self) -> usize {
        if self.bins_x <= self.bins_y {
            self.hist_x.hist().iter().sum()
        } else {
            self.hist_y.hist().iter().sum()
        }
    }

    /// # returns heatmap
    /// * each vector entry will contain the number of times, the corresponding bin was hit
    /// * an entry is 0 if it was never hit
    /// # Access indices; understanding how the data is mapped
    /// A specific heatmap location (x,y)
    /// corresponds to the index `y * self.bins_x() + x`
    pub fn heatmap(&self) -> &Vec<usize>
    {
        &self.heatmap
    }

    /// # returns normalized heatmap
    /// * returns normalized heatmap as Vector 
    /// * Vector contains only f64::NAN, if nothing was in the heatmap
    /// * otherwise the sum of this Vector is 1.0 (or at least very close to 1.0)
    /// # Access indices; understanding how the data is mapped
    /// A specific heatmap location (x,y)
    /// corresponds to the index `y * self.bins_x() + x`
    /// # Note
    /// * used by `self.gnuplot` if option `HeatmapNormalization::NormalizeTotal` is used
    pub fn heatmap_normalized(&self) -> Vec<f64>
    {
        let total = self.total();
        if total == 0 {
            vec![f64::NAN; self.heatmap.len()]
        } else {
            let total = total as f64;
            let mut res = Vec::with_capacity(self.heatmap.len());

            res.extend(
                self.heatmap.iter()
                    .map(|&val| val as f64 / total)
            );

            res
        }
    }

    /// # returns heatmap, normalized column wise
    /// * returns normalized heatmap as Vector 
    /// * Vector contains only f64::NAN, if nothing was in the heatmap
    /// * otherwise the sum of each column (fixed x) will be 1.0, if it contained at least one hit.
    ///  If it did not, the column will only consist of f64::NANs
    /// # Access indices; understanding how the data is mapped
    /// A specific heatmap location (x,y)
    /// corresponds to the index `y * self.bins_x() + x`
    /// # Note
    /// * used by `self.gnuplot` if option `HeatmapNormalization::NormalizeColumn` is used
    pub fn heatmap_normalize_columns(&self) -> Vec<f64>
    {
        let total = self.total();
        let mut res = vec![f64::NAN; self.heatmap.len()];
        if total == 0 {
            return res;
        }
        for x in 0..self.bins_x {
            let column_sum: usize = (0..self.bins_y)
                .map(|y| unsafe{self.get_unchecked(x, y)})
                .sum();

            if column_sum > 0 {
                let denominator = column_sum as f64;
                for y in 0..self.bins_y {
                    let index = self.index(x, y);
                    unsafe {
                        *res.get_unchecked_mut(index) = *self.heatmap.get_unchecked(index) as f64 / denominator;
                    }
                }
            }
        }
        res
    }

    /// # returns heatmap, normalized row wise
    /// * returns normalized heatmap as Vector 
    /// * Vector contains only f64::NAN, if nothing was in the heatmap
    /// * otherwise the sum of each row (fixed y) will be 1.0, if it contained at least one hit.
    ///  If it did not, the row will only consist of f64::NANs
    /// # Access indices; understanding how the data is mapped
    /// A specific heatmap location (x,y)
    /// corresponds to the index `y * self.bins_x() + x`
    /// # Note
    /// * used by `self.gnuplot` if option `HeatmapNormalization::NormalizeRow` is used
    pub fn heatmap_normalize_rows(&self) -> Vec<f64>
    {
        let total = self.total();
        let mut res = vec![f64::NAN; self.heatmap.len()];
        if total == 0 {
            return res;
        }
        for y in 0..self.bins_y {
            let column_sum: usize = (0..self.bins_x)
                .map(|x| unsafe{self.get_unchecked(x, y)})
                .sum();

            if column_sum > 0 {
                let denominator = column_sum as f64;
                for x in 0..self.bins_x {
                    let index = self.index(x, y);
                    unsafe {
                        *res.get_unchecked_mut(index) = *self.heatmap.get_unchecked(index) as f64 / denominator;
                    }
                }
            }
        }
        res
    }
}

impl<HistX, HistY> Heatmap<HistX, HistY>
where 
    HistX: Histogram + std::fmt::Debug,
    HistY: Histogram + std::fmt::Debug,

{
    pub fn count<X, Y>(&mut self, x_val: X, y_val: Y) -> Result<(), HeatmapError>
    where 
        HistX: HistogramVal<X>,
        HistY: HistogramVal<Y>
    {
        let x = self.hist_x
            .get_bin_index(x_val)
            .map_err(|e| {
                    self.error_count += 1;
                    HeatmapError::XError(e)
                }
            )?;
        let y = self.hist_y
            .get_bin_index(y_val)
            .map_err(|e| {
                self.error_count += 1;
                HeatmapError::YError(e)
            }
        )?;
        
        let index = self.index(x, y);
        debug_assert!(index < self.heatmap.len());
        unsafe{
            *self.heatmap.get_unchecked_mut(index) += 1;
        }
        self.hist_x.count_index(x)
            .unwrap();
        self.hist_y.count_index(y)
            .unwrap();

        Ok(())
    }

    fn write_heatmap<W, V, I>(&self, mut data_file: W, iter: I) -> std::io::Result<()>
    where W: Write,
        I: Iterator<Item=V>,
        V: std::fmt::Display
    {
        for (index, val) in iter.enumerate(){
            if (index + 1) % self.bins_x != 0 {
                write!(data_file, "{} ", val)?;
            }else{
                writeln!(data_file, "{}", val)?;
            }
        }
        Ok(())
    }

    pub fn write_heatmap_normalized<W: Write>(&self, data_file: W, mode: HeatmapNormalization) -> std::io::Result<()>
    {
        match mode {
            HeatmapNormalization::AsIs => {
                self.write_heatmap(data_file, self.heatmap.iter())
            },
            HeatmapNormalization::NormalizeTotal => {
                self.write_heatmap(data_file, self.heatmap_normalized().into_iter())
            },
            HeatmapNormalization::NormalizeColumn => {
                self.write_heatmap(data_file, self.heatmap_normalize_columns().into_iter())
            },
            HeatmapNormalization::NormalizeRow => {
                self.write_heatmap(data_file, self.heatmap_normalize_rows().into_iter())
            }
        }
    }

    /// # Plot your heatmap!
    /// This function writes a file, that can be plottet via the terminal via [gnuplot](http://www.gnuplot.info/)
    /// ```bash
    /// gnuplot gnuplot_file
    /// ```
    /// ## Parameter:
    /// * `gnuplot_file`: filename/Path of the file to be plotted. The corresponding file will be truncated, if it already exists
    /// * `gnuplot_output_name`: how shall the file, created by executing gnuplot, be called? Ending of file will be set automatically
    /// * `heatmap_data`: filename/Path of the file where the heatmap data is stored. Needed for plotting the heatmap.
    /// * `normalization_mode`: Should the heatmap be normalized? If yes, how?
    /// ```
    /// use rand_pcg::Pcg64;
    /// use rand::{SeedableRng, distributions::*};
    /// use net_ensembles::sampling::*;
    /// 
    /// let h_x = HistUsizeFast::new_inclusive(0, 10).unwrap();
    /// let h_y = HistU8Fast::new_inclusive(0, 10).unwrap();
    ///
    /// let mut heatmap = Heatmap::new(h_x, h_y);
    /// heatmap.count(0, 0).unwrap();
    /// heatmap.count(10, 0).unwrap();
    ///
    /// let mut rng = Pcg64::seed_from_u64(27456487);
    /// let x_distr = Uniform::new_inclusive(0, 10_usize);
    /// let y_distr = Uniform::new_inclusive(0, 10_u8);
    ///
    /// for _ in 0..100000 {
    ///     let x = x_distr.sample(&mut rng);
    ///     let y = y_distr.sample(&mut rng);
    ///     heatmap.count(x, y).unwrap();
    /// }
    ///
    /// heatmap.gnuplot(
    ///     "heatmap.gp",
    ///     "heatmap",
    ///     "heatmap_data",
    ///     HeatmapNormalization::NormalizeColumn,
    ///     GnuplotTerminal::PDF,
    /// ).unwrap();
    /// ```
    pub fn gnuplot<Path1, Path2, S>(
        &self,
        gnuplot_file: Path1,
        gnuplot_output_name: S,
        data_file: Path2,
        normalization_mode: HeatmapNormalization,
        terminal: GnuplotTerminal
    ) -> std::io::Result<()>
    where 
        Path1: AsRef<Path>,
        Path2: AsRef<Path>,
        S: AsRef<str>
    {
        let gnu = File::create(gnuplot_file)?;
        let mut gnu = BufWriter::new(gnu);

        let data_file_name = data_file.as_ref().to_str().unwrap().to_owned();
        let data = File::create(data_file)?;
        let data = BufWriter::new(data);

        self.write_heatmap_normalized(data, normalization_mode)?;

        writeln!(gnu, "{}", terminal.terminal())?;

        let gnu_out = terminal.output(gnuplot_output_name.as_ref());
        writeln!(gnu, "set output \"{}\"", &gnu_out)?;

        writeln!(gnu, "set xrange[-0.5:{}]", self.bins_x as f64 - 0.5)?;
        writeln!(gnu, "set yrange[-0.5:{}]", self.bins_y as f64 - 0.5)?;

        writeln!(gnu, "set palette model HSV")?;
        writeln!(gnu, "set palette negative defined  ( 0 0 1 0, 2.8 0.4 0.6 0.8, 5.5 0.83 0 1 )")?;
        writeln!(gnu, "set view map")?;

        writeln!(gnu, "set rmargin screen 0.8125\nset lmargin screen 0.175")?;

        writeln!(gnu, "splot \"{}\" matrix with image t \"\" ", data_file_name)?;
        writeln!(gnu, "set output")?;

        terminal.finish(&gnu_out, gnu)
    }

}

#[cfg(test)]
mod tests{
    use rand_pcg::Pcg64;
    use crate::rand::{SeedableRng, distributions::*};
    use super::*;

    #[test]
    fn plot_test()
    {
        let h_x = HistUsizeFast::new_inclusive(0, 10).unwrap();
        let h_y = HistU8Fast::new_inclusive(0, 10).unwrap();

        let mut heatmap = Heatmap::new(h_x, h_y);

        let mut rng = Pcg64::seed_from_u64(27456487);
        let x_distr = Uniform::new_inclusive(0, 10_usize);
        let y_distr = Uniform::new_inclusive(0, 10_u8);

        for _ in 0..100000 {
            let x = x_distr.sample(&mut rng);
            let y = y_distr.sample(&mut rng);
            heatmap.count(x, y).unwrap();
        }

        heatmap.gnuplot(
            "heatmap1.gp",
            "heatmap1",
            "heatmap_data1",
            HeatmapNormalization::NormalizeRow,
            GnuplotTerminal::PDF,
        ).unwrap();

        let normed = heatmap.heatmap_normalize_columns();
        for x in 0..heatmap.bins_x() {
            let mut sum = 0.0;
            for y in 0..heatmap.bins_y()
            {
                sum += normed[heatmap.index(x, y)];
            }
            assert!((sum - 1.0).abs() < 1e-10);
        }


        let normed = heatmap.heatmap_normalize_rows();
        for y in 0..heatmap.bins_y() {
            let mut sum = 0.0;
            for x in 0..heatmap.bins_x()
            {
                sum += normed[heatmap.index(x, y)];
            }
            assert!((sum - 1.0).abs() < 1e-10);
        }
    }

}
