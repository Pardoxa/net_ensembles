use crate::sampling::*;
use std::cmp::*;
use std::io::Write;
use std::fmt::Display;
use glue_helper::*;


/// # Glued together probability
/// * you can [write it to a file](#method.write), maybe the file makes more sense
/// * If you have problems understanding the fields, 
/// please, first look at the [documentation of the current master branch](https://pardoxa.github.io/net_ensembles/master/doc/net_ensembles/)
/// to see, if the documentation there makes more sense. If it doesn't:
///  **open an issue on the github repository**
#[derive(Debug, Clone)]
pub struct GlueResult<T>{
    /// # probably the result you want, i.e., what you were simulating for
    /// * this is the log10 of the probability of each bin
    /// * you get the probability of a bin by `10_f64.powf(glued_log10_probability[i])`, i.e., 10^bin_value
    /// * **normed** such that the sum of the probabilities is 1.0 (within numerical precision errors)
    pub glued_log10_probability: Vec<f64>,
    /// # This are the bin borders
    /// * bin_i is defined as the interval [`borders[i]`, `borders[i + 1]`[, i.e. half open interval
    /// * where the right border is exclusive
    pub borders: Vec<T>,
    /// # log10 of "probabilities" of the curves, you were glueing together
    /// * height adjusted, such that the intervals fit together, but not normalized, the sum can be anything
    pub log10_vec: Vec<Vec<f64>>,
    /// # Index map
    /// * `self.left_list[i]` is the index of `self.borders` where the interval `self.log10_vec`
    /// has the first entry
    pub left_list: Vec<usize>
}

impl<T> GlueResult<T>
where T: Display
{
    /// # Write the result to a file
    /// * for plotting with gnuplot etc. 
    pub fn write<W: Write>(&self, mut w: W) -> std::io::Result<()>
    {
        write!(w, "#bin_left bin_right glued_log_density")?;
        for i in 0..self.log10_vec.len(){
            write!(w, " curve_{}", i)?;
        }
        writeln!(w)?;
        let glue_log_density = &self.glued_log10_probability;
        let borders = &self.borders;
        let log10_vec = &self.log10_vec;
        let left_list = &self.left_list;

        for i in 0..glue_log_density.len(){
            write!(w, "{} {} {:e}", borders[i], borders[i + 1], glue_log_density[i])?;
            for j in 0..log10_vec.len()
            {
                let val = if left_list[j] <= i
                {
                    log10_vec[j].get(i - left_list[j])
                }else {
                    None
                };
                match val {
                    Some(v) => write!(w, " {:e}", v)?,
                    None => write!(w, " NONE")?,
                };
            }
            writeln!(w)?;
        }
        Ok(())
    }
}

/// # Combine multiple WangLandau intervals to get the probability distribution of the whole interval
/// * `list`: Vector of Wang landau distributions. Is mutable, because the list will be sorted.
/// Appart from that, this list will not be changed and can be used by you later on without problems
/// # Restrictions
/// * `original_hist` has to contain all the borders of the histograms.
/// Meaning: The size of a bin has to be constant among all histograms of the `list`.
/// If it is not, you might get an error, or you might get wrong results. 
/// **I do not check for this exaustingly**.
/// * There is an **easy way** to make sure, that you don`t get problems here:
/// Create the `original_hist` first. Then create the other Histograms using the `HistogramPartion` trait.
/// This is the intended way. But as long as the borders and bin sizes match, this function will work properly
/// # Understanding returned Parameters (OK(..)):
pub fn glue_wl<WL, HIST, T>(list: &mut Vec<WL>, original_hist: &HIST) -> Result<GlueResult<T>, GlueErrors>
where WL: WangLandauHist<HIST>,
    HIST: Histogram + HistogramVal<T>,
    T: PartialOrd
{
    if list.is_empty(){
        return Err(GlueErrors::EmptyList);
    }

    // sort
    list.sort_unstable_by(|a, b| {
            a.hist()
                .get_left()
                .partial_cmp(
                    &b.hist()
                    .get_left()
                ).unwrap_or(Ordering::Less)
        }
    );
    
    let borders = original_hist.borders_clone()
        .map_err(|e| GlueErrors::BorderCreation(e))?;

    let mut left_list = Vec::with_capacity(list.len());
    let mut right_list = Vec::with_capacity(list.len());
    for wl in list.iter()
    {
        left_list.push(get_index(&wl.hist().get_left(), &borders)?);
        right_list.push(get_index(&wl.hist().get_right(), &borders)?);
    }

    // get log densitys
    let mut log10_vec: Vec<_> = list.iter()
        .map(|wl| wl.log_density_base10())
        .collect();

    // re-normalize to prevent later precision problems
    re_normalize_density(&mut log10_vec);

    // calc z
    let z_vec = calc_z(&log10_vec, &left_list, &right_list)?;

    // correct height
    height_correction(&mut log10_vec, &z_vec);

    // glueing together
    let mut glue_log_density = glue(original_hist.bin_count(), &log10_vec, &left_list, &right_list)?;

    // now norm the result
    norm_sum_to_1(&mut glue_log_density);
    
    let glue_res = GlueResult{
        log10_vec,
        glued_log10_probability: glue_log_density,
        left_list,
        borders,
    };
    Ok(glue_res)
}
