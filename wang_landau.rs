use serde::{Serialize, Deserialize};
use net_ensembles::{rand::{Rng, seq::*}, *};
use std::{marker::PhantomData, iter::*};
use crate::wang_landau::*;
use std::{collections::*, cmp::*};


struct ProbIndex{
    index: usize,
    diff: f64
}

impl PartialEq for ProbIndex{
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
        &&
        self.diff == other.diff
    }
}

impl Eq for ProbIndex{}

impl PartialOrd for ProbIndex{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.diff.partial_cmp(&other.diff)
    }
}

impl Ord for ProbIndex{
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl ProbIndex{
    fn new(prob: f64, index: usize) -> Self
    {
        debug_assert!(prob.is_finite());
        Self{
            index,
            diff: (0.5 - prob).copysign(-1.0)
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum WangLandauMode {
    RefineOriginal,
    Refine1T
}

impl WangLandauMode{
    pub fn is_mode_original(&self) -> bool {
        match self {
            WangLandauMode::RefineOriginal => true,
            _ => false
        }
    }

    pub fn is_mode_1_t(&self) -> bool {
        match self {
            WangLandauMode::Refine1T => true,
            _ => false
        }
    }
}

/// # Adaptive WangLandau 1/t
/// * **please cite** 
/// > Yannick Feld and Alexander K. Hartmann,
/// > “Large-deviations of the basin stability of power grids,”
/// > *Chaos*&nbsp;**29**:113113&nbsp;(2019), DOI&nbsp;[10.1063/1.5121415](https://dx.doi.org/10.1063/1.5121415)
///
/// as this adaptive approach was first used and described in this paper. Also cite the following
/// * The 1/t Wang Landau approach comes from this paper
/// > R. E. Belardinelli and V. D. Pereyra,
/// > Fast algorithm to calculate density of states,”
/// > Phys.&nbsp;Rev.&nbsp;E&nbsp;**75**: 046701 (2007), DOI&nbsp;[10.1103/PhysRevE.75.046701](https://doi.org/10.1103/PhysRevE.75.046701)
/// 
/// * The original Wang Landau algorithim comes from this paper
/// > F. Wang and D. P. Landau,
/// > “Efficient, multiple-range random walk algorithm to calculate the density of states,” 
/// > Phys.&nbsp;Rev.&nbsp;Lett.&nbsp;**86**, 2050–2053 (2001), DOI&nbsp;[10.1103/PhysRevLett.86.2050](https://doi.org/10.1103/PhysRevLett.86.2050)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WangLandauAdaptive<R, E, S, Res, Hist, T>
{
    rng: R,
    samples_per_trial: usize,
    trial_list: Vec<usize>,
    best_of_steps: Vec<usize>,
    best_of_count: usize,
    ensemble: E,
    step_marker: PhantomData::<S>,
    step_res_marker: PhantomData<Res>,
    accepted_step_hist: Vec<usize>,
    rejected_step_hist: Vec<usize>,
    min_step: usize,
    counter: usize,
    log_f: f64,
    log_f_theshold: f64,
    step_count: usize,
    histogram: Hist,
    log_density: Vec<f64>,
    old_energy: T,
    old_bin: usize,
    mode: WangLandauMode,
    check_refine_every: usize,
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
    NotEnoughStatistics,
    /// Still Gathering Statistics, this is only an estimate!
    EstimatedStatistic(Vec<f64>),
    /// bestof has to be at least 1 and at most the number of distinct steps tried,
    /// i.e., max_step - min_step + 1
    InvalidBestof,

    /// check refine has to be at least 1
    CheckRefineEvery0
}

impl<R, E, S, Res, Hist, T> WangLandauAdaptive<R, E, S, Res, Hist, T>
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

    pub fn hist(&self) -> &Hist
    {
        &self.histogram
    }

    fn statistic_bin_not_hit(&self) -> bool
    {
        self.accepted_step_hist.iter()
            .zip(self.rejected_step_hist.iter())
            .any(|(a,b )| a+b == 0)
    }

    pub fn create_statistics(&self) -> Result<Vec<f64>, WangLandauErrors>
    {
        let calc_estimate = || {
            let estimate: Vec<_> = self.accepted_step_hist
            .iter()
            .zip(
                self.rejected_step_hist.iter()
            ).map(|(&a, &r)|
                {
                    a as f64 / (a + r) as f64
                }
            ).collect();
            estimate
        };
        if self.is_gathering_statistics() {
            
            if self.statistic_bin_not_hit()
            {
                Err(WangLandauErrors::NotEnoughStatistics)
            } else{
                
                Err(WangLandauErrors::EstimatedStatistic(calc_estimate()))
            }
        } else {
            Ok(
                calc_estimate()
            ) 
        }
    }

    pub fn get_old_energy(&self) -> &T
    {
        &self.old_energy
    }


    /// **panics** if index is invalid
    pub fn metropolis_acception_prob(&self, old_bin: usize, new_bin: usize) -> f64
    {
        1.0f64.min(
            (self.log_density[old_bin] - self.log_density[new_bin])
                .exp()
        )
    }
    
}

impl<R, E, S, Res, Hist, T> WangLandauAdaptive<R, E, S, Res, Hist, T> 
where R: Rng,
    E: MarkovChain<S, Res>,
    Hist: Histogram + HistogramVal<T>
{
    fn log_f_1_t(&self) -> f64 
    {
        self.hist().bin_count() as f64 / self.step_count as f64
    }

    fn reset_statistics(&mut self)
    {
        self.best_of_steps.clear();
        self.counter = 0;
    }

    fn adjust_bestof(&mut self){
        self.best_of_steps.clear();
        self.generate_bestof();
    }

    fn generate_bestof(&mut self)
    {
        let statistics = self.create_statistics().unwrap();
        let mut heap: BinaryHeap<_> = statistics.into_iter()
            .enumerate()
            .map(|(index, prob)|
                {
                    ProbIndex::new(prob, index)
                }
            ).collect();
        while self.best_of_steps.len() < self.best_of_count
        {
            let index = heap.pop().unwrap().index;
            let step_size = index + self.min_step;
            self.best_of_steps.push(step_size);
        }
    }

    fn get_stepsize(&mut self) -> usize {
        match self.trial_list.get(self.counter) {
            None => {
                if self.best_of_steps.is_empty(){
                    self.generate_bestof();
                }
                *self.best_of_steps.choose(&mut self.rng).unwrap()
            },
            Some(step_size) => *step_size,
        }
    }

    fn check_refine(&mut self)
    {
        match self.mode{
            WangLandauMode::Refine1T => {
                self.log_f = self.log_f_1_t();
                let adjust = 2000.max(4 * self.check_refine_every);
                if self.step_count % adjust == 0 {
                    self.adjust_bestof();
                }
                return;
            },
            WangLandauMode::RefineOriginal => {
                if self.step_count % self.check_refine_every == 0 && !self.histogram.any_bin_zero() {
                    let ref_1_t = self.log_f_1_t();
                    self.log_f *= 0.5;
                    if self.log_f < ref_1_t {
                        self.log_f = ref_1_t;
                        self.mode = WangLandauMode::Refine1T;
                    } else {
                        self.reset_statistics();
                    }
                }
            }
        }
    }
}


impl<R, E, S, Res, Hist, T> WangLandauAdaptive<R, E, S, Res, Hist, T> 
where R: Rng,
    E: MarkovChain<S, Res>,
    Hist: Histogram + HistogramVal<T>,
    T: Default
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
        trial_step_max: usize,
        best_of_count: usize,
        histogram: Hist,
        check_refine_every: usize
    ) -> Result<Self, WangLandauErrors>
    {
        if trial_step_max < trial_step_min
        {
            return Err(WangLandauErrors::InvalidMinMaxTrialSteps);
        } 
        else if !log_f_theshold.is_finite() || log_f_theshold.is_sign_negative() 
        {
            return Err(WangLandauErrors::InvalidLogFThreshold);
        }else if check_refine_every == 0 {
            return Err(WangLandauErrors::CheckRefineEvery0)
        }

        let distinct_step_count = trial_step_max - trial_step_min + 1;

        if best_of_count > distinct_step_count {
            return Err(WangLandauErrors::InvalidBestof);
        }

        let mut trial_list = Vec::with_capacity(distinct_step_count * samples_per_trial);
        trial_list.extend (
            (trial_step_min..=trial_step_max)
                .flat_map(|s| repeat(s).take(samples_per_trial))
        );
        
        trial_list.shuffle(&mut rng);
        
        
        let accepted_step_hist = vec![0; distinct_step_count];
        let rejected_step_hist = vec![0; distinct_step_count];

        let log_density = vec![0.0; histogram.bin_count()]; 

        Ok(
            Self{
                ensemble,
                counter: 0,
                min_step: trial_step_min,
                accepted_step_hist,
                rejected_step_hist,
                trial_list,
                rng,
                samples_per_trial,
                step_marker: PhantomData::<S>,
                step_res_marker: PhantomData::<Res>,
                log_f: 1.0,
                log_f_theshold,
                step_count: 0,
                histogram,
                log_density,
                old_energy: T::default(),
                mode: WangLandauMode::RefineOriginal,
                old_bin: usize::MAX,
                best_of_count,
                best_of_steps: Vec::with_capacity(best_of_count),
                check_refine_every
            }
        )
    }

    pub fn wang_landau_convergence<F, G>(
        &mut self,
        energy_fn: F,
        valid_ensemble: G
    )where F: Fn(&mut E) -> T + Copy,
        G: Fn(&E) -> bool + Copy,
    {
        while self.log_f > self.log_f_theshold{
            self.wang_landau_step(energy_fn, valid_ensemble);
        }
    }

    pub fn wang_landau_step<F, G>(
        &mut self,
        energy_fn: F,
        valid_ensemble: G
    )where F: Fn(&mut E) -> T,
        G: Fn(&E) -> bool,
    {
        self.step_count += 1;
        let step_size = self.get_stepsize();

        let steps = self.ensemble.m_steps(step_size);
        
        if !valid_ensemble(&self.ensemble)
        {
            self.rejected_step_hist[step_size - self.min_step] += 1;
            self.counter += 1;
            
            self.histogram.count_index(self.old_bin).unwrap();
            self.ensemble.undo_steps_quiet(steps);
            return;
        }
        
        self.check_refine();
        
        let current_energy = energy_fn(&mut self.ensemble);
        let current_bin = match self.histogram.get_bin_index(&current_energy)
        {
            Ok(index) => index,
            _  => {
                // invalid step
                
                self.rejected_step_hist[step_size - self.min_step] += 1;
                self.counter += 1;
                
                self.histogram.count_index(self.old_bin).unwrap();
                self.log_density[self.old_bin] += self.log_f;
                self.ensemble.undo_steps_quiet(steps);
                return;
            }
        };
        let accept_prob = self.metropolis_acception_prob(self.old_bin, current_bin);

        if self.rng.gen::<f64>() > accept_prob {
            // reject step
            self.rejected_step_hist[step_size - self.min_step] += 1;
            self.counter += 1;

            self.histogram.count_index(self.old_bin).unwrap();
            self.log_density[self.old_bin] += self.log_f;
            self.ensemble.undo_steps_quiet(steps);
            return;
        } else {
            // reject step
            self.accepted_step_hist[step_size - self.min_step] += 1;
            self.counter += 1;
            
            self.old_energy = current_energy;
            self.old_bin = current_bin;
            self.histogram.count_index(current_bin).unwrap();
            self.log_density[current_bin] += self.log_f;
            return;
        }
    }
}
