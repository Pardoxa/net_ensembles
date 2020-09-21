use crate::sampling::*;
use std::borrow::*;
use transpose::*;
use std::io::Write;

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};


/// # Get index of heatmap corresponding to a coordinate
#[inline(always)]
pub fn heatmap_index(width: usize, x: usize, y: usize) -> usize
{
    y * width + x
}

/// # Heatmap
/// * stores heatmap in row-major order: the rows of the heatmap are contiguous,
/// and the columns are strided
/// * enables you to quickly create a heatmap
/// * you can create gnuplot scripts to plot the heatmap
/// * you can transpose the heatmap
/// * …
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct HeatmapUsize<HistWidth, HistHeight>{
    pub(crate) hist_width: HistWidth,
    pub(crate) hist_height: HistHeight,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) heatmap: Vec<usize>, // stored width, height
    pub(crate) error_count: usize
}

pub type HeatmapU<HistWidth, HistHeight> = HeatmapUsize<HistWidth, HistHeight>;

impl <HistWidth, HistHeight> HeatmapUsize<HistWidth, HistHeight>
where 
    HistWidth: Clone,
    HistHeight: Clone,
{
    /// # Use this to get a "flipped" heatmap
    /// * creates a transposed heatmap
    /// * also look at [`self.transpose_inplace`](#method.transpose_inplace)
    pub fn transpose(&self) -> HeatmapUsize<HistHeight, HistWidth>
    {
        let mut transposed = vec![0; self.heatmap.len()];
        transpose(
            &self.heatmap,
            &mut transposed,
            self.width,
            self.height
        );
        HeatmapUsize{
            hist_width: self.hist_height.clone(),
            hist_height: self.hist_width.clone(),
            width: self.height,
            height: self.width,
            error_count: self.error_count,
            heatmap: transposed,
        }
    }
}

impl <HistWidth, HistHeight> HeatmapUsize<HistWidth, HistHeight>
{

    /// # Use this to get a "flipped" heatmap
    /// * transposes the heatmap inplace
    pub fn transpose_inplace(mut self) -> HeatmapUsize<HistHeight, HistWidth>
    {
        let mut scratch = vec![0; self.width.max(self.height)];
        transpose_inplace(&mut self.heatmap, &mut scratch, self.width, self.height);
        HeatmapUsize{
            hist_width: self.hist_height,
            hist_height: self.hist_width,
            width: self.height,
            height: self.width,
            error_count: self.error_count,
            heatmap: self.heatmap
        }
    }

    /// x = j
    /// y = i
    #[inline(always)]
    fn index(&self, x: usize, y: usize) -> usize
    {
        heatmap_index(self.width, x, y)
    }

    /// Returns value stored in the heatmap at specified 
    /// coordinates, or `None`, if out of Bounds
    pub fn get(&self, x: usize, y: usize) -> Option<usize>
    {
        self.heatmap.get(self.index(x, y)).copied()
    }

    /// # row of the heatmap
    /// * `None` if out of bounds
    /// * otherwise it is a slice of the row at height `y`
    /// # Note
    /// *  there is no `get_column` method, because, due to implementation details,
    /// it is way less efficient, and could not be returned as slice
    pub fn get_row(&self, y: usize) -> Option<&[usize]>
    {
        let fin = self.index(self.width, y);
        if fin > self.heatmap.len(){
            None
        } else {
            let start = fin - self.width;
            Some(
                &self.heatmap[start..fin]
            )
        }
    }

    /// Returns value stored in the heatmap at specified 
    /// coordinates without performing bound checks.
    /// **undefined behavior** if coordinates are out of bounds
    pub unsafe fn get_unchecked(&self, x: usize, y: usize) -> usize
    {
        *self.heatmap.get_unchecked(self.index(x, y))
    }

    /// # returns width of the heatmap
    /// * the width is the same size, as the `self.width_projection().bin_count()` 
    pub fn width(&self) -> usize
    {
        self.width
    }

    /// # returns height of the heatmap
    /// * the height is the same size, as the `self.height_projection().bin_count()` 
    pub fn height(&self) -> usize
    {
        self.height
    } 


    /// # Returns reference to current width Histogram
    /// * histogram used to bin in the "width" direction
    /// * all `counts` are counted here -> this is a projection of the heatmap
    pub fn width_hist(&self) -> &HistWidth{
        &self.hist_width
    }

    /// # Returns reference to current height Histogram
    /// * histogram used to bin in the "height" direction
    /// * all `counts` are counted here -> this is a projection of the heatmap
    pub fn height_hist(&self) -> &HistHeight{
        &self.hist_height
    }
}


impl<HistWidth, HistHeight> HeatmapUsize<HistWidth, HistHeight>
where 
    HistWidth: Histogram,
    HistHeight: Histogram,
{

    /// # Create a new Heatmap
    /// * heatmap will have width `width_hist.bin_count()` 
    /// and height `height_hist.bin_count()`
    /// * histograms will be reset (zeroed) here, so it does not matter, if they 
    /// were used before and contain Data
    pub fn new(mut width_hist: HistWidth, mut height_hist: HistHeight) -> Self {
        let width = width_hist.bin_count();
        let height = height_hist.bin_count();
        width_hist.reset();
        height_hist.reset();
        let heatmap = vec![0; width * height];
        Self{
            width,
            height,
            heatmap,
            hist_width: width_hist,
            hist_height: height_hist,
            error_count: 0
        }
    }

    /// # Reset
    /// * resets histograms 
    /// * heatmap is reset to contain only 0's
    /// * miss_count is reset to 0
    pub fn reset(&mut self)
    {
        self.hist_width.reset();
        self.hist_height.reset();
        self.heatmap.iter_mut().for_each(|v| *v = 0);
        self.error_count = 0;
    }

    /// # "combine" heatmaps
    /// * heatmaps will be combined by adding all entrys of `other` to `self`
    /// * heatmaps have to have the same dimensions
    pub fn combine<OtherHW, OtherHH>(&mut self, other: &HeatmapUsize<OtherHW, OtherHH>) -> Result<(), HeatmapError>
    where OtherHW: Histogram,
        OtherHH: Histogram,
    {
        if self.width != other.width || self.height != other.height
        {
            return Err(HeatmapError::Dimension);
        }
        self.heatmap
            .iter_mut()
            .zip(
                other.heatmap.iter()
            ).for_each(
                |(this, other)|
                {
                    *this += other;
                }
            );
        
        for (i, &count) in other.hist_width.hist().iter().enumerate()
        {
            self.hist_width
                .count_multiple_index(i, count)
                .unwrap()
        }

        for (i, &count) in other.hist_height.hist().iter().enumerate()
        {
            self.hist_height
                .count_multiple_index(i, count)
                .unwrap()
        }
        self.error_count += other.error_count;
        
        Ok(())
    }

    /// # counts how often the heatmap was hit
    /// * should be equal to `self.heatmap.iter().sum::<usize>()` but more efficient
    /// * Note: it calculates this in O(min(self.width, self.height))
    pub fn total(&self) -> usize {
        if self.width <= self.height {
            self.hist_width.hist().iter().sum()
        } else {
            self.hist_height.hist().iter().sum()
        }
    }

    /// check if at least one bin was hit
    fn any_hit(&self) -> bool {

        let hist_vec = 
        if self.width <= self.height {
            self.hist_width
                .hist()
        } else {
            self.hist_height
                .hist()
        };

        hist_vec
                .iter()
                .any(|&val| val != 0)
    }

    /// # Counts how often the Heatmap was missed, i.e., you tried to count a value (x,y), which was outside the Heatmap
    pub fn total_misses(&self) -> usize
    {
        self.error_count
    }

    /// # counts how many bins of the heatmap where hit at least once
    pub fn bins_hit(&self) -> usize
    {
        self.heatmap
            .iter()
            .filter(|&&val| val > 0)
            .count()
    }

    /// # counts how many bins of the heatmap where never hit
    pub fn bins_not_hit(&self) -> usize
    {
        self.heatmap
            .iter()
            .filter(|&&val| val == 0)
            .count()
    }

    /// # returns heatmap
    /// * each vector entry will contain the number of times, the corresponding bin was hit
    /// * an entry is 0 if it was never hit
    /// # Access indices; understanding how the data is mapped
    /// * A specific heatmap location `(x,y)`
    /// corresponds to the index `y * self.width() + x`
    /// * you can use the `heatmap_index` function to calculate the index
    pub fn heatmap(&self) -> &Vec<usize>
    {
        &self.heatmap
    }


    /// # returns Vector representing normalized heatmap
    /// * Vector contains only 0.0, if nothing was in the heatmap
    /// * otherwise the sum of this Vector is 1.0 (or at least very close to 1.0)
    /// # Access indices; understanding how the data is mapped
    /// * A specific heatmap location (x,y)
    /// corresponds to the index `y * self.width() + x`
    /// * you can use the function `heatmap_index(width, x, y)` for calculating the index
    pub fn vec_normalized(&self) -> Vec<f64>
    {
        let total = self.total();
        
        if total == 0 {
            vec![0.0; self.heatmap.len()]
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

    /// # returns normalized heatmap
    /// * returns normalized heatmap as `HeatmapF64` 
    /// * Heatmap vector `self.heatmap_normalized().heatmap()` contains only 0.0, if nothing was in the heatmap
    /// * otherwise the sum of this Vector is 1.0 (within numerical errors)
    pub fn heatmap_normalized(&self) -> HeatmapF64<HistWidth, HistHeight>
    where HistHeight: Clone,
        HistWidth: Clone
    {
        let heatmap_vec = self.vec_normalized();

        HeatmapF64{
            heatmap: heatmap_vec,
            hist_height: self.hist_height.clone(),
            hist_width: self.hist_width.clone(),
            error_count: self.error_count,
            width: self.width,
            height: self.height
        }
    }

    /// # returns normalized heatmap
    /// * returns normalized heatmap as `HeatmapF64` 
    /// * Heatmap vector `self.heatmap_normalized().heatmap()` contains only 0.0, if nothing was in the heatmap
    /// * otherwise the sum of this Vector is 1.0 (within numerical errors)
    pub fn into_heatmap_normalized(self) -> HeatmapF64<HistWidth, HistHeight>
    {
        let heatmap_vec = self.vec_normalized();

        HeatmapF64{
            heatmap: heatmap_vec,
            hist_height: self.hist_height,
            hist_width: self.hist_width,
            error_count: self.error_count,
            width: self.width,
            height: self.height
        }
    }

    
    /// # returns vector representing heatmap, normalized column wise
    /// * Vector contains only 0.0, if nothing was in the heatmap
    /// * otherwise the sum of each column (fixed x) will be 1.0 (within numerical errors), if it contained at least one hit.
    ///  If it did not, the column will only consist of 0.0
    /// # Access indices; understanding how the data is mapped
    /// A specific heatmap location (x,y)
    /// corresponds to the index `y * self.width() + x`
    /// * you can use the function `heatmap_index(width, x, y)` for calculating the index
    pub fn vec_normalized_columns(&self) -> Vec<f64>
    {
        
        let mut res = vec![0.0; self.heatmap.len()];
        if !self.any_hit() {
            return res;
        }
        for x in 0..self.width {
            let column_sum: usize = (0..self.height)
                .map(|y| unsafe{self.get_unchecked(x, y)})
                .sum();

            if column_sum > 0 {
                let denominator = column_sum as f64;
                for y in 0..self.height {
                    let index = self.index(x, y);
                    unsafe {
                        *res.get_unchecked_mut(index) = *self.heatmap.get_unchecked(index) as f64 / denominator;
                    }
                }
            }
        }
        res
    }

    /// # returns (column wise) normalized heatmap
    /// * returns normalized heatmap as `HeatmapF64` 
    /// * Heatmap vector `self.heatmap_normalized().heatmap()` contains only 0.0, if nothing was in the heatmap
    /// * otherwise the sum of each column (fixed x) will be 1.0 (within numerical errors), if it contained at least one hit.
    ///  If it did not, the column will only consist of 0.0
    /// * otherwise the sum of this Vector is 1.0 
    pub fn heatmap_normalized_columns(&self) -> HeatmapF64<HistWidth, HistHeight>
    where HistHeight: Clone,
        HistWidth: Clone
    {
        let heatmap_vec = self.vec_normalized_columns();

        HeatmapF64{
            heatmap: heatmap_vec,
            hist_height: self.hist_height.clone(),
            hist_width: self.hist_width.clone(),
            error_count: self.error_count,
            width: self.width,
            height: self.height
        }
    }

    /// # returns (column wise) normalized heatmap
    /// * returns normalized heatmap as `HeatmapF64` 
    /// * Heatmap vector `self.heatmap_normalized().heatmap()` contains only 0.0, if nothing was in the heatmap
    /// * otherwise the sum of each column (fixed x) will be 1.0 (within numerical errors), if it contained at least one hit.
    ///  If it did not, the column will only consist of 0.0
    /// * otherwise the sum of this Vector is 1.0 
    pub fn into_heatmap_normalized_columns(self) -> HeatmapF64<HistWidth, HistHeight>
    {
        let heatmap_vec = self.vec_normalized_columns();

        HeatmapF64{
            heatmap: heatmap_vec,
            hist_height: self.hist_height,
            hist_width: self.hist_width,
            error_count: self.error_count,
            width: self.width,
            height: self.height
        }
    }

    /// # returns vector representing heatmap, normalized row wise
    /// * Vector contains only 0.0, if nothing was in the heatmap
    /// * otherwise the sum of each row (fixed x) will be 1.0 (within numerical errors), if it contained at least one hit.
    ///  If it did not, the row will only consist of 0.0
    /// # Access indices; understanding how the data is mapped
    /// A specific heatmap location (x,y)
    /// corresponds to the index `y * self.width() + x`
    /// * you can use the function `heatmap_index(width, x, y)` for calculating the index
    pub fn vec_normalized_rows(&self) -> Vec<f64>
    {
        
        let mut res = vec![0.0; self.heatmap.len()];
        if !self.any_hit() {
            return res;
        }
        for y in 0..self.height {
            let start_index = self.index(0, y);
            let fin = start_index + self.width;
            let row_slice = &self.heatmap[start_index..fin];
            let row_sum = row_slice.iter()
                .sum::<usize>();

            if row_sum > 0 {
                let denominator = row_sum as f64;
                let res_slice = &mut res[start_index..fin];
                for (res_val, &heat_val) in res_slice
                    .into_iter()
                    .zip(row_slice.into_iter())
                {
                    *res_val = heat_val as f64 / denominator;
                }
            }
        }
        res
    }

    /// # returns (row wise) normalized heatmap
    /// * returns normalized heatmap as `HeatmapF64` 
    /// * Heatmap vector `self.heatmap_normalized().heatmap()` contains only 0.0, if nothing was in the heatmap
    /// * otherwise the sum of each row (fixed x) will be 1.0 (within numerical errors), if it contained at least one hit.
    ///  If it did not, the row will only consist of 0.0
    /// * otherwise the sum of this Vector is 1.0 
    pub fn heatmap_normalized_rows(&self) -> HeatmapF64<HistWidth, HistHeight>
    where HistHeight: Clone,
        HistWidth: Clone
    {
        let heatmap_vec = self.vec_normalized_rows();

        HeatmapF64{
            heatmap: heatmap_vec,
            hist_height: self.hist_height.clone(),
            hist_width: self.hist_width.clone(),
            error_count: self.error_count,
            width: self.width,
            height: self.height
        }
    }

    /// # returns (row wise) normalized heatmap
    /// * returns normalized heatmap as `HeatmapF64` 
    /// * Heatmap vector `self.heatmap_normalized().heatmap()` contains only 0.0, if nothing was in the heatmap
    /// * otherwise the sum of each row (fixed x) will be 1.0 (within numerical errors), if it contained at least one hit.
    ///  If it did not, the row will only consist of 0.0
    /// * otherwise the sum of this Vector is 1.0 
    pub fn into_heatmap_normalized_rows(self) -> HeatmapF64<HistWidth, HistHeight>
    {
        let heatmap_vec = self.vec_normalized_rows();

        HeatmapF64{
            heatmap: heatmap_vec,
            hist_height: self.hist_height,
            hist_width: self.hist_width,
            error_count: self.error_count,
            width: self.width,
            height: self.height
        }
    }

    /// # update the heatmap
    /// * calculates the coordinate `(x, y)` of the bin corresponding
    /// to the given value pair `(width_val, height_val)`
    /// * if coordinate is out of bounds, it counts a "miss" and returns the HeatmapError
    /// * otherwise it counts the "hit" and returns the coordinate `(x, y)`
    pub fn count<A, B, X, Y>(&mut self, width_val: A, height_val: B) -> Result<(usize, usize), HeatmapError>
    where 
        HistWidth: HistogramVal<X>,
        HistHeight: HistogramVal<Y>,
        A: Borrow<X>,
        B: Borrow<Y>
    {
        let x = self.hist_width
            .get_bin_index(width_val)
            .map_err(|e| {
                    self.error_count += 1;
                    HeatmapError::XError(e)
                }
            )?;
        let y = self.hist_height
            .count_val(height_val)
            .map_err(|e| {
                self.error_count += 1;
                HeatmapError::YError(e)
            }
        )?;
        
        let index = self.index(x, y);
        unsafe{
            *self.heatmap.get_unchecked_mut(index) += 1;
        }

        self.hist_width
            .count_index(x)
            .unwrap();

        Ok((x, y))
    }

    /// # Write heatmap to file
    /// * writes data of heatmap to file.
    /// # file
    /// * lets assume `self.width()`is 4 and `self.height()` is 3
    /// * the resulting file could look like
    /// ```txt
    /// 0 1 0 10
    /// 100 0 0 1
    /// 2 9 1 0
    /// ```
    pub fn write_to<W>(&self, mut data_file: W) -> std::io::Result<()>
    where W: Write
    {
        for y in 0..self.height {
            let row = self.get_row(y).unwrap();

            if let Some((last, slice)) = row.split_last() {
                for val in slice {
                    write!(data_file, "{} ", val)?;
                }
                writeln!(data_file, "{}", last)?;
            }
        }
        Ok(())
    }

    /// # Create a gnuplot script to plot your heatmap
    /// This function writes a file, that can be plottet via the terminal via [gnuplot](http://www.gnuplot.info/)
    /// ```bash
    /// gnuplot gnuplot_file
    /// ```
    /// ## Parameter:
    /// * `gnuplot_writer`: writer gnuplot script will be written to
    /// * `gnuplot_output_name`: how shall the file, created by executing gnuplot, be called? Ending of file will be set automatically
    /// ## Example
    /// ```
    /// use rand_pcg::Pcg64;
    /// use rand::{SeedableRng, distributions::*};
    /// use net_ensembles::sampling::*;
    /// use std::fs::File;
    /// use std::io::BufWriter;
    /// 
    /// // first randomly create a heatmap
    /// let h_x = HistUsizeFast::new_inclusive(0, 10).unwrap();
    /// let h_y = HistU8Fast::new_inclusive(0, 10).unwrap();
    ///
    /// let mut heatmap = HeatmapU::new(h_x, h_y);
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
    /// // create File for gnuplot skript
    /// let file = File::create("heatmap.gp").unwrap();
    /// let buf = BufWriter::new(file);
    ///
    /// // Choose settings for gnuplot
    /// let mut settings = GnuplotSettings::new();
    /// settings.x_axis(GnuplotAxis::new(-5.0, 5.0, 6))
    ///     .y_axis(GnuplotAxis::from_slice(&["a", "b", "c", "d"]))
    ///     .y_label("letter")
    ///     .x_label("number")
    ///     .title("Example");
    ///
    /// // create skript
    /// heatmap.gnuplot(
    ///     buf,
    ///     "heatmap.pdf",
    ///     settings
    /// ).unwrap();
    /// ```
    /// Skript can now be plotted with
    /// ```bash
    /// gnuplot heatmap.gp
    /// ```
    pub fn gnuplot<W, S, GS>(
        &self,
        mut gnuplot_writer: W,
        gnuplot_output_name: S,
        settings: GS
    ) -> std::io::Result<()>
    where 
        W: Write,
        S: AsRef<str>,
        GS: Borrow<GnuplotSettings>
    {
        let settings = settings.borrow();
        settings.terminal_str();
        writeln!(gnuplot_writer, "{}", settings.terminal_str())?;
        write!(gnuplot_writer, "set output \"")?;
        settings.terminal.output(gnuplot_output_name.as_ref(), &mut gnuplot_writer)?;
        writeln!(gnuplot_writer, "\"")?;
        settings.write_label(&mut gnuplot_writer)?;


        writeln!(gnuplot_writer, "set xrange[-0.5:{}]", self.width as f64 - 0.5)?;
        writeln!(gnuplot_writer, "set yrange[-0.5:{}]", self.height as f64 - 0.5)?;
        if !settings.title.is_empty(){
            writeln!(gnuplot_writer, "set title '{}'", settings.title)?;
        }

        settings.write_axis(
            &mut gnuplot_writer,
            self.hist_width.bin_count(),
            self.hist_height.bin_count()
        )?;

        settings.pallet.write_pallet(&mut gnuplot_writer)?;
        writeln!(gnuplot_writer, "set view map")?;

        writeln!(gnuplot_writer, "set rmargin screen 0.8125\nset lmargin screen 0.175")?;
        writeln!(gnuplot_writer, "$data << EOD")?;
        self.write_to(&mut gnuplot_writer)?;
        writeln!(gnuplot_writer, "EOD")?;
        writeln!(gnuplot_writer, "splot $data matrix with image t \"{}\" ", settings.get_title())?;

        writeln!(gnuplot_writer, "set output")?;

        settings.terminal.finish(gnuplot_output_name.as_ref(), gnuplot_writer)
    }

}

#[cfg(test)]
mod tests{
    use rand_pcg::Pcg64;
    use crate::rand::{SeedableRng, distributions::*};
    use super::*;

    #[test]
    fn row_test()
    {
        let h_x = HistUsizeFast::new_inclusive(0, 10).unwrap();
        let h_y = HistU8Fast::new_inclusive(0, 6).unwrap();

        let mut heatmap = HeatmapUsize::new(h_x, h_y);

        let mut rng = Pcg64::seed_from_u64(27456487);
        let x_distr = Uniform::new_inclusive(0, 10_usize);
        let y_distr = Uniform::new_inclusive(0, 6_u8);

        for _ in 0..100 {
            let x = x_distr.sample(&mut rng);
            let y = y_distr.sample(&mut rng);
            heatmap.count(x, y).unwrap();
        }

        let mut iter = heatmap.heatmap().iter();
        for y in 0..heatmap.height()
        {
            let row = heatmap.get_row(y).unwrap();
            assert_eq!(row.len(), heatmap.width());
            for val in row
            {
                assert_eq!(val, iter.next().unwrap());
            }
        }
    }

    #[test]
    fn combine_test()
    {
        let h_x = HistUsizeFast::new_inclusive(0, 10).unwrap();
        let h_y = HistU8Fast::new_inclusive(0, 6).unwrap();

        let mut heatmap = HeatmapUsize::new(h_x, h_y);

        let mut rng = Pcg64::seed_from_u64(27456487);
        let x_distr = Uniform::new_inclusive(0, 10_usize);
        let y_distr = Uniform::new_inclusive(0, 6_u8);

        for _ in 0..100 {
            let x = x_distr.sample(&mut rng);
            let y = y_distr.sample(&mut rng);
            heatmap.count(x, y).unwrap();
        }

        let c = heatmap.clone();
        heatmap.combine(&c).unwrap();

    }

    #[test]
    fn plot_test()
    {
        let h_x = HistUsizeFast::new_inclusive(0, 10).unwrap();
        let h_y = HistU8Fast::new_inclusive(0, 10).unwrap();

        let mut heatmap = HeatmapUsize::new(h_x, h_y);

        let mut rng = Pcg64::seed_from_u64(27456487);
        let x_distr = Uniform::new_inclusive(0, 10_usize);
        let y_distr = Uniform::new_inclusive(0, 10_u8);

        for _ in 0..100000 {
            let x = x_distr.sample(&mut rng);
            let y = y_distr.sample(&mut rng);
            heatmap.count(x, y).unwrap();
        }

        // heatmap.gnuplot(
        //     "EPS.gp",
        //     "EPS",
        //     "EPS_DATA",
        //     HeatmapNormalization::NormalizeRow,
        //     GnuplotTerminal::EpsLatex,
        // ).unwrap();

        for x in 0..heatmap.width() {
            let mut sum = 0;
            for y in 0..heatmap.height()
            {
                sum += heatmap.get(x, y).unwrap();
            }
            assert_eq!(sum, heatmap.width_hist().hist()[x]);
        }

        for y in 0..heatmap.height() {
            let mut sum = 0;
            for x in 0..heatmap.width()
            {
                sum += heatmap.get(x, y).unwrap();
            }
            assert_eq!(sum, heatmap.height_hist().hist()[y]);
        }

        let normed = heatmap.vec_normalized_columns();
        for x in 0..heatmap.width() {
            let mut sum = 0.0;
            for y in 0..heatmap.height()
            {
                sum += normed[heatmap.index(x, y)];
            }
            assert!((sum - 1.0).abs() < 1e-10);
        }


        let normed = heatmap.vec_normalized_rows();
        for y in 0..heatmap.height() {
            let mut sum = 0.0;
            for x in 0..heatmap.width()
            {
                sum += normed[heatmap.index(x, y)];
            }
            assert!((sum - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn transpose_test()
    {
        let h_x = HistUsizeFast::new_inclusive(0, 10).unwrap();
        let h_y = HistU8Fast::new_inclusive(0, 5).unwrap();

        let mut heatmap = HeatmapUsize::new(h_x, h_y);

        let mut rng = Pcg64::seed_from_u64(27456487);
        let x_distr = Uniform::new_inclusive(0, 10_usize);
        let y_distr = Uniform::new_inclusive(0, 5_u8);

        for _ in 0..10 {
            let x = x_distr.sample(&mut rng);
            let y = y_distr.sample(&mut rng);
            heatmap.count(x, y).unwrap();
        }

        // heatmap.gnuplot(
        //     "heatmapT.gp",
        //     "heatmapT",
        //     "heatmap_dataT",
        //     HeatmapNormalization::AsIs,
        //     GnuplotTerminal::PDF,
        // ).unwrap();

        let heatmap_t = heatmap.transpose();

        // heatmap_t.gnuplot(
        //     "heatmapT_T.gp",
        //     "heatmapT_T",
        //     "heatmap_dataT_T",
        //     HeatmapNormalization::AsIs,
        //     GnuplotTerminal::PDF,
        // ).unwrap();

        let heatmap_i = heatmap.transpose_inplace();

        // heatmap_i.gnuplot(
        //     "heatmapT_I.gp",
        //     "heatmapT_I",
        //     "heatmap_dataT_I",
        //     HeatmapNormalization::AsIs,
        //     GnuplotTerminal::PDF,
        // ).unwrap();

        for (val1, val2) in heatmap_i.heatmap().iter().zip(heatmap_t.heatmap().iter())
        {
            assert_eq!(val1, val2);
        }

        for x in 0..heatmap_i.width() {
            let mut sum = 0;
            for y in 0..heatmap_i.height()
            {
                sum += heatmap_i.get(x, y).unwrap();
            }
            assert_eq!(sum, heatmap_i.width_hist().hist()[x]);
        }

        for y in 0..heatmap_i.height() {
            let mut sum = 0;
            for x in 0..heatmap_i.width()
            {
                sum += heatmap_i.get(x, y).unwrap();
            }
            assert_eq!(sum, heatmap_i.height_hist().hist()[y]);
        }
    }

}
