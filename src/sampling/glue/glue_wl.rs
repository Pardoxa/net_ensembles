use crate::sampling::*;
use std::cmp::*;
use std::io::Write;
use std::fmt::Display;
use glue_helper::*;


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
/// * The first 
pub fn glue_wl<WL, HIST, T>(list: &mut Vec<WL>, original_hist: &HIST) -> Result<(Vec<f64>, Vec<T>, Vec<Vec<f64>>, Vec<usize>), GlueErrors>
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

    // get log densitys
    let mut log10_vec: Vec<_> = list.iter()
        .map(|wl| wl.log_density_base10())
        .collect();
    
    // re-normalize to prevent later precision problems
    re_normalize_density(&mut log10_vec);
    
    let borders = original_hist.borders_clone()
        .map_err(|e| GlueErrors::BorderCreation(e))?;

    let mut left_list = Vec::with_capacity(list.len());
    let mut right_list = Vec::with_capacity(list.len());
    for wl in list.iter()
    {
        left_list.push(get_index(&wl.hist().get_left(), &borders)?);
        right_list.push(get_index(&wl.hist().get_right(), &borders)?);
    }

    // calc z
    let z_vec = calc_z(&log10_vec, &left_list, &right_list)?;

    // correct height
    height_correction(&mut log10_vec, &z_vec);

    // glueing together
    let mut glue_log_density = glue(original_hist.bin_count(), &log10_vec, &left_list, &right_list)?;

    // now norm the result
    norm_sum_to_1(&mut glue_log_density);
    
    Ok((glue_log_density, borders, log10_vec, left_list))
}

pub fn glue_wl_write<WL, HIST, T, W>(list: &mut Vec<WL>, original_hist: &HIST, mut w: W) -> Result<(Vec<f64>, Vec<T>, Vec<Vec<f64>>, Vec<usize>), GlueErrors>
where WL: WangLandauHist<HIST>,
    HIST: Histogram + HistogramVal<T> + HistogramPartition,
    T: PartialOrd + Display,
    W: Write,
{
    let (glue_log_density, borders, log10_vec, left_list) = glue_wl(list, original_hist)?;
    write!(w, "#bin_left bin_right glued_log_density")?;
    for i in 0..log10_vec.len(){
        write!(w, " curve_{}", i)?;
    }
    writeln!(w)?;

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

    Ok((glue_log_density, borders, log10_vec, left_list))
}