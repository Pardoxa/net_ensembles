use crate::sampling::*;
use std::cmp::*;
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
pub fn glue_entropic<Entr, HIST, T>(list: &mut Vec<Entr>, original_hist: &HIST) -> Result<GlueResult<T>, GlueErrors>
where Entr: EntropicHist<HIST>,
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