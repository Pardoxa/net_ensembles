use crate::sampling::*;
use std::cmp::*;
use std::convert::*;
use std::io::Write;
use std::fmt::Display;

#[derive(Debug)]
pub enum GlueErrors{
    BorderCreation(HistErrors),
    EmptyList,
    BinarySearch,
    OutOfBounds,
    NoOverlap,
    IO(std::io::Error)
}

impl From<HistErrors> for GlueErrors{
    fn from(e: HistErrors) -> Self {
        GlueErrors::BorderCreation(e)
    }
}

impl From<std::io::Error> for GlueErrors{
    fn from(e: std::io::Error) -> Self {
        GlueErrors::IO(e)
    }
}

pub fn glue_wl<WL, HIST, T>(list: &mut Vec<WL>, original_hist: &HIST) -> Result<(Vec<f64>, Vec<T>, Vec<Vec<f64>>, Vec<usize>), GlueErrors>
where WL: WangLandauHist<HIST>,
    HIST: Histogram + HistogramVal<T> + HistogramPartition,
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
    log10_vec.iter_mut()
        .for_each(
            |v|
            {
                let max = v.iter().copied().fold(f64::NAN, f64::max);
                if max.is_finite() {
                    v.iter_mut()
                        .for_each(|val| *val -= max);
                }
            }
        );
    let mut glue_log_density = vec![f64::NAN;original_hist.bin_count()];
    let borders = original_hist.borders_clone()
        .map_err(|e| GlueErrors::BorderCreation(e))?;

    let mut left_list = Vec::with_capacity(list.len());
    let mut right_list = Vec::with_capacity(list.len());
    for wl in list.iter()
    {
        left_list.push(get_index(&wl.hist().get_left(), &borders)?);
        right_list.push(get_index(&wl.hist().get_right(), &borders)?);
    }


    // init 
    let first_log = log10_vec.first().unwrap();
    let l = *left_list.first().unwrap();
    let r = *right_list.first().unwrap();
    if r >= glue_log_density.len() {
        return Err(GlueErrors::OutOfBounds);
    }
    glue_log_density[l..=r].copy_from_slice(first_log);
    let mut glue_count = vec![0_usize; glue_log_density.len()];
    for i in l..=r {
        glue_count[i] = 1;
    }


    // calc z
    let mut z_vec = Vec::with_capacity(list.len() - 1);
    for i in 1..list.len()
    {
            let left_prev = left_list[i - 1];
            let left = left_list[i];
            let l_m = left.max(left_prev);
            let right_prev = right_list[i - 1];
            let right = right_list[i];
            let r_m = right.min(right_prev);
            if l_m >= r_m {
                return Err(GlueErrors::NoOverlap);
            }
            let overlap_size = r_m - l_m;
            let (prev, cur) = if left_prev >= left{
                let diff = left_prev - left;
                (
                    &log10_vec[i - 1][0..=overlap_size],
                    &log10_vec[i][diff..=diff+overlap_size]
                )
            } else {
                let diff = left - left_prev;
                (
                    &log10_vec[i - 1][diff..=diff+overlap_size],
                    &log10_vec[i][0..=overlap_size]
                )
            };
            let sum = prev.iter().zip(cur.iter())
                .fold(0.0, |acc, (&p, &c)| p - c + acc);
            let mut z = sum / prev.len() as f64;
            // also correct for adjustment of prev
            if let Some(val) = z_vec.last() {
                z += val;
            }
            z_vec.push(z);
    }

    // correct height
    log10_vec.iter_mut()
        .skip(1)
        .zip(z_vec.iter())
        .for_each( |(vec, &z)|
            vec.iter_mut()
                .for_each(|val| *val += z )
        );

    
    for (i, log_vec) in log10_vec.iter().enumerate().skip(1)
    {
        let left = left_list[i];
        let right = right_list[i];
        glue_log_density[left..=right].iter_mut()
            .zip(glue_count[left..=right].iter_mut())
            .zip(log_vec.iter())
            .for_each(|((res, count), &val)| {
                *count += 1;
                if res.is_finite(){
                    *res += val;
                } else {
                    *res = val;
                }
            });
    }

    glue_log_density.iter_mut()
        .zip(glue_count.iter())
        .for_each(|(log, &count)| {
            if count > 0 {
                *log /= count as f64;
            }
        });

    // now norm the result
    let sum = glue_log_density.iter()
        .fold(0.0, |acc, &val| {
            if val.is_finite(){
               acc +  10_f64.powf(val)
            } else {
                acc
            }
        }  );
    
    let sum = sum.log10();
    glue_log_density.iter_mut()
        .for_each(|val| *val -= sum);
    
    Ok((glue_log_density, borders, log10_vec, left_list))
}

fn get_index<T>(val: &T, borders: &Vec<T>) -> Result<usize, GlueErrors>
where 
    T: PartialOrd
{
    let mut error = false;
    let index = borders.binary_search_by(
        |probe|{
            probe.partial_cmp(val).unwrap_or_else(|| {
                    error = true;
                    Ordering::Equal
                }
            )
        }
    );
    if error {
        return Err(GlueErrors::BinarySearch);
    }
    index.map_err(|_| GlueErrors::BinarySearch)
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
