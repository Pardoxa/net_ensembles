use rand::{Rng, seq::*};
use average::Variance;


/// returns reduced value + estimated error (as sqrt of variance)
pub fn bootstrap<F, R, N1>(mut rng: R, samples: usize, data: &[N1], reduction: F) -> (f64, f64)
where F: Fn (&[&N1]) -> f64,
    R: Rng,
{
    let mut bootstrap_sample = Vec::with_capacity(data.len());
    let variance: Variance =
    (0..samples).map(|_|
        {
            bootstrap_sample.clear();
            bootstrap_sample.extend(
                (0..data.len())
                    .map(|_| 
                        {
                            data.choose(&mut rng).unwrap()
                        }
                    )
            );
            let reduced = reduction(&bootstrap_sample);
            reduced
        }
    ).collect();
    
    let mean = variance.mean();
    let variance = variance.population_variance();
    (mean, variance)
}

/// Similar to ```bootstrap``` but for stuff that implements `Copy`. Likely more effient in these cases
/// returns reduced value + estimated error (as sqrt of variance)
pub fn bootstrap_copyable<F, R, N1>(mut rng: R, samples: usize, data: &[N1], reduction: F) -> (f64, f64)
where F: Fn (&mut [N1]) -> f64,
    R: Rng,
    N1: Copy
{
    let mut bootstrap_sample = Vec::with_capacity(data.len());
    let variance: Variance =
    (0..samples).map(|_|
        {
            bootstrap_sample.clear();
            bootstrap_sample.extend(
                (0..data.len())
                    .map(|_| 
                        {
                            data.choose(&mut rng).unwrap()
                        }
                    )
            );
            let reduced = reduction(&mut bootstrap_sample);
            reduced
        }
    ).collect();
    
    let mean = variance.mean();
    let variance = variance.population_variance();
    (mean, variance)
}