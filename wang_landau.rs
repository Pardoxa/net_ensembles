use serde::{Serialize, Deserialize};
use net_ensembles::{rand::{Rng, seq::*}, *};
use std::{marker::PhantomData, iter::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WangLandauAdaptive<R, E, S, Res>
{
    rng: R,
    samples_per_trial: usize,
    trial_list: Vec<usize>,
    ensemble: E,
    step_marker: PhantomData::<S>,
    step_res_marker: PhantomData<Res>,
    good_step_hist: Vec<usize>,
    bad_step_hist: Vec<usize>,
    min_step: usize,
    counter: usize,
    log_f: f64,
    log_f_theshold: f64,
    step_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WangLandauErrors{
    /// `trial_step_min <= trial_step_max` has to be true
    InvalidMinMaxTrialSteps,
    /// `log_f_theshold`can never be negative or zero!
    /// it also cannot be NaN or infinite!
    InvalidLogFThreshold,
    /// Still in the process of gathering statistics
    /// Not enough to make an estimate
    NotEnoughStatitsics,
    /// Still Gathering Statistics, this is only an estimate!
    EstimatedStatistic(Vec<f64>)
}

impl<R, E, S, Res> WangLandauAdaptive<R, E, S, Res>
{
    /// returns currently set threshold
    #[inline]
    pub fn log_f_theshold(&self) -> f64
    {
        self.log_f_theshold
    }

    /// Try to set the threshold. 
    /// * `log_f_theshold > 0.0` has to be true
    /// * `log_f_theshold` has to be finite
    pub fn set_log_f_theshold(&mut self, log_f_theshold: f64) -> Result<f64, WangLandauErrors>
    {
        if !log_f_theshold.is_finite() || log_f_theshold.is_sign_negative() {
            return Err(WangLandauErrors::InvalidLogFThreshold);
        }
        let old_theshold = self.log_f_theshold;
        self.log_f_theshold = log_f_theshold;
        Ok(old_theshold)
    }

    /// get current value of log_f
    #[inline]
    pub fn log_f(&self) -> f64
    {
        self.log_f
    }

    /// how many steps were performed until now?
    #[inline]
    pub fn get_step_count(&self) -> usize
    {
        self.step_count
    }

    /// Is the simulation in the process of gathering statistics,
    /// i.e., is it currently trying many differnt step sizes?
    #[inline]
    pub fn is_gathering_statistics(&self) -> bool
    {
        self.counter < self.trial_list.len()
    }

    /// returns value between 0 and 1.0
    pub fn fraction_of_statistics_gathered(&self) -> f64
    {
        let frac = self.counter as f64 / self.trial_list.len() as f64;
        if frac > 1.0 {
            1.0
        } else {
            frac
        }
    }

    /// access to current state of your ensemble
    pub fn ensemble(&self) -> &E
    {
        &self.ensemble
    }

    pub fn create_statistics(&self) -> Result<Vec<f64>, WangLandauErrors>
    {
        if self.is_gathering_statistics() {
            let hist: Vec<_> = 
            self.good_step_hist
                .iter()
                .zip(
                    self.bad_step_hist
                        .iter()
                ).map(
                    |(good, bad)|
                    {
                        good + bad
                    }
                )
                .collect();
            if hist.iter().any(|&val| val == 0)
            {
                Err(WangLandauErrors::NotEnoughStatitsics)
            } else{
                let estimate: Vec<_> = self.good_step_hist
                    .iter()
                    .zip(
                        hist.iter()
                    ).map(|(&good, &total)|
                        {
                            good as f64 / total as f64
                        }
                    ).collect();
                Err(WangLandauErrors::EstimatedStatistic(estimate))
            }
        } else {
            Ok(
                self.good_step_hist.iter()
                .map(|&good|
                    {
                        good as f64 / self.samples_per_trial as f64
                    }
                ).collect()
            ) 
        }
    }
}


impl<R, E, S, Res> WangLandauAdaptive<R, E, S, Res> 
where R: Rng,
    E: MarkovChain<S, Res>
{
   
    /// # important:
    /// * **Err** if `trial_step_max < trial_step_min`
    /// * **Err** if `log_f_theshold <= 0.0`
    pub fn new(
        log_f_theshold: f64,
        ensemble: E, 
        mut rng: R, 
        samples_per_trial: usize, 
        trial_step_min: usize, 
        trial_step_max: usize
    ) -> Result<Self, WangLandauErrors>
    {
        if trial_step_max < trial_step_min
        {
            return Err(WangLandauErrors::InvalidMinMaxTrialSteps);
        } 
        else if !log_f_theshold.is_finite() || log_f_theshold.is_sign_negative() 
        {
            return Err(WangLandauErrors::InvalidLogFThreshold);
        }

        let distinct_step_count = trial_step_max - trial_step_min + 1;

        let mut trial_list = Vec::with_capacity(distinct_step_count * samples_per_trial);
        trial_list.extend (
            (trial_step_min..=trial_step_max)
                .flat_map(|s| repeat(s).take(samples_per_trial))
        );
        
        trial_list.shuffle(&mut rng);
        
        
        let good_step_hist = vec![0; distinct_step_count];
        let bad_step_hist = vec![0; distinct_step_count];

        Ok(
            Self{
                ensemble,
                counter: 0,
                min_step: trial_step_min,
                good_step_hist,
                bad_step_hist,
                trial_list,
                rng,
                samples_per_trial,
                step_marker: PhantomData::<S>,
                step_res_marker: PhantomData::<Res>,
                log_f: 1.0,
                log_f_theshold,
                step_count: 0,
            }
        )
    }
}