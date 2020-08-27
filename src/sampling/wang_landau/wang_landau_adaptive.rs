use crate::{rand::{Rng, seq::*}, *};
use std::{marker::PhantomData, iter::*, io::Write};
use crate::sampling::*;
use std::{collections::*, cmp::*};
use num_traits::{Bounded, ops::wrapping::*, identities::*};

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

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
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct WangLandauAdaptive<Hist, R, E, S, Res, Energy>
{
    pub(crate) rng: R,
    pub(crate) samples_per_trial: usize,
    pub(crate) trial_list: Vec<usize>,
    pub(crate) best_of_steps: Vec<usize>,
    pub(crate) min_best_of_count: usize,
    pub(crate) best_of_threshold: f64,
    pub(crate) ensemble: E,
    pub(crate) step_marker: PhantomData::<S>,
    pub(crate) step_res_marker: PhantomData<Res>,
    pub(crate) accepted_step_hist: Vec<usize>,
    pub(crate) rejected_step_hist: Vec<usize>,
    total_steps_rejected: usize,
    total_steps_accepted: usize,
    pub(crate) min_step: usize,
    counter: usize,
    log_f: f64,
    log_f_threshold: f64,
    pub(crate) step_count: usize,
    pub(crate) histogram: Hist,
    pub(crate) log_density: Vec<f64>,
    pub(crate) old_energy: Option<Energy>,
    pub(crate) old_bin: Option<usize>,
    mode: WangLandauMode,
    pub(crate) check_refine_every: usize,
}


impl<R, E, S, Res, Hist, T> WangLandau for WangLandauAdaptive<Hist, R, E, S, Res, T>
{
    #[inline(always)]
    fn log_f(&self) -> f64
    {
        self.log_f
    }

    #[inline(always)]
    fn log_f_threshold(&self) -> f64
    {
        self.log_f_threshold
    }

    fn set_log_f_threshold(&mut self, log_f_threshold: f64) -> Result<f64, WangLandauErrors>
    {
        if !log_f_threshold.is_finite() || log_f_threshold.is_sign_negative() {
            return Err(WangLandauErrors::InvalidLogFThreshold);
        }
        let old_threshold = self.log_f_threshold;
        self.log_f_threshold = log_f_threshold;
        Ok(old_threshold)
    }

    #[inline(always)]
    fn log_density(&self) -> &Vec<f64>
    {
        &self.log_density
    }

    #[inline(always)]
    fn mode(&self) -> WangLandauMode
    {
        self.mode
    }

    #[inline(always)]
    fn step_counter(&self) -> usize
    {
        self.step_count
    }

    fn write_log<W: Write>(&self, mut writer: W) -> Result<(), std::io::Error> {
        writeln!(writer,
            "#Acceptance prob_total: {}\n#Acceptance prob current: {}\n#total_steps: {}\n#log_f: {:e}\n#Current_mode {:?}",
            self.fraction_accepted_total(),
            self.fraction_accepted_current(),
            self.step_counter(),
            self.log_f(),
            self.mode
        )?;

        writeln!(writer, "#min_step_size {}", self.min_step_size())?;
        writeln!(writer, "#max_step_size {}", self.max_step_size())?;

        write!(writer, "#Current acception histogram:")?;
        for val in self.accepted_step_hist.iter()
        {
            write!(writer, " {}", val)?;
        }

        write!(writer, "\n#Current rejection histogram:")?;
        for val in self.rejected_step_hist.iter()
        {
            write!(writer, " {}", val)?;
        }

        writeln!(writer, "\n#bestof threshold: {}", self.best_of_threshold)?;
        writeln!(writer, "#min_bestof_count: {}", self.min_best_of_count)?;
        write!(writer, "\n#Current_Bestof:")?;

        for val in self.best_of_steps.iter()
        {
            write!(writer, " {}", val)?;
        }

        write!(writer, "#current_statistics_estimate:")?;
        let estimate = self.estimate_statistics();
        match estimate {
            Ok(estimate) => {
                for val in estimate
                {
                    write!(writer, " {}", val)?;
                }
                writeln!(writer)
            },
            _ => {
                writeln!(writer, " None")
            }
        }
    }
}

impl<R, E, S, Res, Hist, T> WangLandauEnsemble<E> 
    for WangLandauAdaptive<Hist, R, E, S, Res, T>
{
    #[inline(always)]
    fn ensemble(&self) -> &E
    {
        &self.ensemble
    }
}

impl<R, E, S, Res, Hist, Energy> WangLandauEnergy<Energy> 
    for WangLandauAdaptive<Hist, R, E, S, Res, Energy>
{
    #[inline(always)]
    fn energy(&self) -> Option<&Energy> {
        self.old_energy.as_ref()
    }
}

impl<R, E, S, Res, Hist, Energy> WangLandauHist<Hist>
    for WangLandauAdaptive<Hist, R, E, S, Res, Energy>
{
    #[inline(always)]
    fn hist(&self) -> &Hist {
        &self.histogram
    }
}

impl<R, E, S, Res, Hist, T> WangLandauAdaptive<Hist, R, E, S, Res, T>
{

    /// # Smallest possible markov step (`m_steps` of MarkovChain trait) tried by wang landau step
    #[inline]
    pub fn min_step_size(&self) -> usize
    {
        self.min_step
    }

    /// # Largest possible markov step (`m_steps` of MarkovChain trait) tried by wang landau step
    #[inline]
    pub fn max_step_size(&self) -> usize 
    {
        self.min_step + self.accepted_step_hist.len() - 1
    }

    /// Is the simulation in the process of rebuilding the statistics,
    /// i.e., is it currently trying many differnt step sizes?
    #[inline]
    pub fn is_rebuilding_statistics(&self) -> bool
    {
        self.counter < self.trial_list.len()
    }

    /// # Tracks progress
    /// * tracks progress until `self.is_rebuilding_statistics` becomes false
    /// * returned value is always `0 <= val <= 1.0`
    pub fn fraction_of_statistics_gathered(&self) -> f64
    {
        let frac = self.counter as f64 / self.trial_list.len() as f64;
        if frac > 1.0 {
            1.0
        } else {
            frac
        }
    }

    /// # total_steps_accepted / total_steps 
    pub fn fraction_accepted_total(&self) -> f64 {
        let total_acc = self.total_steps_accepted 
            + self.accepted_step_hist
                .iter()
                .sum::<usize>();
        let total_steps = total_acc + self.total_steps_rejected 
            + self.rejected_step_hist
                .iter()
                .sum::<usize>();

        if total_steps == 0 {
            f64::NAN
        } else {
            total_acc as f64 / total_steps as f64
        }
    }

    /// # Fraction of steps accepted since the statistics were reset the last time
    /// * (steps accepted since last reset) / (steps since last reset)
    pub fn fraction_accepted_current(&self) -> f64 {
        let accepted: usize = self.accepted_step_hist.iter().sum();
        let total = accepted + self.rejected_step_hist.iter().sum::<usize>();
        if total == 0 {
            f64::NAN
        } else {
            accepted as f64 / total as f64
        }
    }

    fn statistic_bin_not_hit(&self) -> bool
    {
        self.accepted_step_hist.iter()
            .zip(self.rejected_step_hist.iter())
            .any(|(a,b )| a+b == 0)
    }

    /// # Estimate accept/reject statistics
    /// * contains list of estimated probabilities for accepting a step of corresponding step size
    /// * list[i] corresponds to step size `i + self.min_step`
    /// * O(trial_step_max - trial_step_min)
    pub fn estimate_statistics(&self) -> Result<Vec<f64>, WangLandauErrors>
    {
        let calc_estimate = || {
            let mut estimate = Vec::with_capacity(self.accepted_step_hist.len());
            estimate.extend(
                self.accepted_step_hist
                    .iter()
                    .zip(
                        self.rejected_step_hist.iter()
                    ).map(|(&a, &r)|
                        {
                            a as f64 / (a + r) as f64
                        }
                    )
            );
            estimate
        };
        if self.is_rebuilding_statistics() {
            
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


    /// **panics** if index is invalid
    fn metropolis_acception_prob(&self, old_bin: usize, new_bin: usize) -> f64
    {
        
        (self.log_density[old_bin] - self.log_density[new_bin])
            .exp()
        
    }
    
}

impl<R, E, S, Res, Hist, T> WangLandauAdaptive<Hist, R, E, S, Res, T> 
where R: Rng,
    E: MarkovChain<S, Res>,
    Hist: Histogram + HistogramVal<T>
{
    
    fn reset_statistics(&mut self)
    {
        self.best_of_steps.clear();

        self.total_steps_accepted += self.accepted_step_hist.iter().sum::<usize>();
        self.accepted_step_hist
            .iter_mut()
            .for_each(|entry| *entry = 0);

        self.total_steps_rejected += self.rejected_step_hist.iter().sum::<usize>();
        self.rejected_step_hist
            .iter_mut()
            .for_each(|entry| *entry = 0);

        self.counter = 0;
    }

    fn adjust_bestof(&mut self){
        self.best_of_steps.clear();
        self.generate_bestof();
    }

    fn generate_bestof(&mut self)
    {
        let statistics = self.estimate_statistics().unwrap();
        let mut heap = BinaryHeap::with_capacity(statistics.len());
        heap.extend(statistics.into_iter()
            .enumerate()
            .map(|(index, prob)|
                {
                    ProbIndex::new(prob, index)
                }
            )
        );
        while let Some(p_i) = heap.pop() {
            if p_i.is_best_of(self.best_of_threshold) 
                || self.best_of_steps.len() < self.min_best_of_count
            {
                let step_size = p_i.index + self.min_step;
                self.best_of_steps.push(step_size);
            } else {
                break;
            }
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

    fn count_accepted(&mut self, size: usize){
        self.accepted_step_hist[size - self.min_step] += 1;
        self.counter += 1;
    }

    fn count_rejected(&mut self, size: usize){
        self.rejected_step_hist[size - self.min_step] += 1;
        self.counter += 1;
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
                    self.histogram.reset();
                }
            }
        }
    }
}


impl<R, E, S, Res, Hist, T> WangLandauAdaptive<Hist, R, E, S, Res, T> 
where R: Rng,
    E: MarkovChain<S, Res>,
    Hist: Histogram + HistogramVal<T>,
    T: Clone
{
   
    /// # New WangLandauAdaptive
    /// * `log_f_threshold` - threshold for the simulation
    /// * `ensemble` ensemble used for the simulation
    /// * `rng` - random number generator used
    /// * `samples_per_trial` - how often a specific step_size should be tried before
    /// estimating the fraction of accepted steps resulting from the stepsize
    /// * `trial_step_min` and `trial_step_max`: The step sizes tried are: [trial_step_min, trial_step_min + 1, ..., trial_step_max]
    /// * `min_best_of_count`: After estimating, use at least the best `min_best_of_count` step sizes found
    /// * `best_of_threshold`: After estimating, use all steps for which abs(acceptance_rate -0.5) <= best_of_threshold holds true
    /// * `histogram`: How your energy will be binned etc
    /// * `check_refine_every`: how often to check if log_f can be refined?
    /// # Important
    /// * **You need to call on of the  `self.init*` members before starting the Wang Landau simulation!
    /// * **Err** if `trial_step_max < trial_step_min`
    /// * **Err** if `log_f_threshold <= 0.0`
    pub fn new(
        log_f_threshold: f64,
        ensemble: E, 
        mut rng: R, 
        samples_per_trial: usize, 
        trial_step_min: usize, 
        trial_step_max: usize,
        min_best_of_count: usize,
        mut best_of_threshold: f64,
        histogram: Hist,
        check_refine_every: usize
    ) -> Result<Self, WangLandauErrors>
    {
        if trial_step_max < trial_step_min
        {
            return Err(WangLandauErrors::InvalidMinMaxTrialSteps);
        } 
        else if !log_f_threshold.is_finite() || log_f_threshold.is_sign_negative() 
        {
            return Err(WangLandauErrors::InvalidLogFThreshold);
        }
        else if check_refine_every == 0 {
            return Err(WangLandauErrors::CheckRefineEvery0)
        }
        if !best_of_threshold.is_finite(){
            best_of_threshold = 0.0;
        }

        let distinct_step_count = trial_step_max - trial_step_min + 1;

        if min_best_of_count > distinct_step_count {
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
                log_f_threshold,
                step_count: 0,
                histogram,
                log_density,
                old_energy: None,
                mode: WangLandauMode::RefineOriginal,
                old_bin: None,
                min_best_of_count,
                best_of_steps: Vec::with_capacity(min_best_of_count),
                check_refine_every,
                total_steps_accepted: 0,
                total_steps_rejected: 0,
                best_of_threshold,
            }
        )
    }


    /// ensures a valid ensemble
    fn init<F>(
        &mut self,
        energy_fn: F,
        step_limit: Option<u64>
    ) -> Result<(), WangLandauErrors>
    where F: Fn(&mut E) -> Option<T>
    {
        
        self.old_energy = energy_fn(&mut self.ensemble);
        if self.old_energy.is_some(){
            return Ok(());
        }
        match step_limit {
            None => {
                loop {
                    let step_size = self.get_stepsize();
                    self.ensemble.m_steps_quiet(step_size);
                    self.old_energy = energy_fn(&mut self.ensemble);
        
                    if self.old_energy.is_some(){
                        self.count_accepted(step_size);
                        return Ok(());
                    }
                    self.count_rejected(step_size);
                }
            },
            Some(limit) => {
                for _ in 0..limit {
                    let step_size = self.get_stepsize();
                    self.ensemble.m_steps_quiet(step_size);
                    self.old_energy = energy_fn(&mut self.ensemble);
        
                    if self.old_energy.is_some(){
                        self.count_accepted(step_size);
                        return Ok(());
                    }
                    self.count_rejected(step_size);
                }
                Err(WangLandauErrors::InitFailed)
            }
        }
        
        
    }

    fn end_init(&mut self)
    {
        self.reset_statistics();
        self.old_bin = self.histogram
            .get_bin_index( 
                self.old_energy_clone()
            ).ok();
        assert!(self.old_bin.is_some(), "Error in heuristic - old bin invalid");
    }

    fn old_energy_clone(&self) -> T {
        self.old_energy
            .as_ref()
            .unwrap()
            .clone()
    }

    fn greedy_helper<F, H, J>(
        &mut self,
        old_distance: &mut J,
        energy_fn: F,
        distance_fn: H
    )   where F: Fn(&mut E) -> Option<T>,
            H: Fn(&Hist, T) -> J,
            J: PartialOrd
    {
        let size = self.get_stepsize();
        let steps = self.ensemble.m_steps(size);

        
        if let Some(energy) = energy_fn(&mut self.ensemble) {
            let distance = distance_fn(&self.histogram, energy.clone());
            if distance <= *old_distance {
                self.old_energy = Some(energy);
                *old_distance = distance;
                self.count_accepted(size);
                return;
            }
        }

        self.count_rejected(size);
        self.ensemble.undo_steps_quiet(steps);
        
    }

    /// # Find a valid starting Point
    /// * if the ensemble is already at a valid starting point,
    /// the ensemble is left unchanged (as long as your energy calculation does not change the ensemble)
    /// * `overlap` - see trait HistogramIntervalDistance. 
    /// Should be greater than 0 and smaller than the number of bins in your histogram. E.g. `overlap = 3` if you have 200 bins
    /// * `mid` - should be something like `128u8`, `0i8` or `0i16`. It is very unlikely that using a type with more than 16 bit makes sense for mid
    /// * `step_limit`: Some(val) -> val is max number of steps tried, if no valid state is found, it will return an Error. None -> will loop until either 
    /// a valid state is found or forever
    /// * alternates between greedy and interval heuristik everytime a wrapping counter passes `mid` or `U::min_value()`
    /// * I recommend using this heuristik, if you do not know which one to use
    /// # Parameter
     /// * `energy_fn` function calculating `Some(energy)` of the system
    /// or rather the Parameter of which you wish to obtain the probability distribution.
    ///  Has to be the same function as used for the wang landau simulation later.
    /// If there are any states, for which the calculation is invalid, `None` should be returned
    /// * steps resulting in ensembles for which `energy_fn(&mut ensemble)` is `None`
    /// will always be rejected 
    pub fn init_mixed_heuristik<F, U>
    (
        &mut self,
        overlap: usize,
        mid: U,
        energy_fn: F,
        step_limit: Option<u64>
    ) -> Result<(), WangLandauErrors>
    where F: Fn(&mut E) -> Option<T>,
        Hist: HistogramIntervalDistance<T>,
        U: One + Bounded + WrappingAdd + Eq + PartialOrd
    {
        let overlap = overlap.max(1);
        self.init(&energy_fn, step_limit)?;
        if self.histogram
            .is_inside(
                self.old_energy
                    .as_ref()
                    .unwrap()
            )
        {
            self.end_init();
            return Ok(());
        }    
        
        let mut old_dist = f64::INFINITY;
        let mut old_dist_interval = usize::MAX;
        let mut counter: U = U::min_value();
        let min_val = U::min_value();
        let one = U::one();
        let dist_interval = |h: &Hist, val: T| h.interval_distance_overlap(val, overlap);
        let mut step_count = 0;
        loop {
            let current_energy = self.old_energy_clone();
            if counter == min_val {
                old_dist = self.histogram.distance(current_energy);
            }else if counter == mid {
                old_dist_interval = dist_interval(&self.histogram, current_energy);
            }
            if counter < mid {
                self.greedy_helper(
                    &mut old_dist,
                    &energy_fn,
                    Hist::distance
                );
                if old_dist == 0.0 {
                    break;
                }
            } else {
                self.greedy_helper(
                    &mut old_dist_interval,
                    &energy_fn,
                    dist_interval
                );
                if old_dist_interval == 0 {
                    break;
                }
            }
            counter = counter.wrapping_add(&one);
            if let Some(limit) = step_limit {
                if limit == step_count{
                    return Err(WangLandauErrors::InitFailed);
                }
                step_count += 1;
            }
        }
        self.end_init();
        Ok(())
    }

    /// # Find a valid starting Point
    /// * if the ensemble is already at a valid starting point,
    /// the ensemble is left unchanged (as long as your energy calculation does not change the ensemble)
    /// * Uses overlapping intervals. Accepts a step, if the resulting ensemble is in the same interval as before,
    /// or it is in an interval closer to the target interval
    /// # Parameter
    /// * `step_limit`: Some(val) -> val is max number of steps tried, if no valid state is found, it will return an Error. None -> will loop until either 
    /// a valid state is found or forever
    /// * `energy_fn` function calculating `Some(energy)` of the system
    /// or rather the Parameter of which you wish to obtain the probability distribution.
    ///  Has to be the same function as used for the wang landau simulation later.
    /// If there are any states, for which the calculation is invalid, `None` should be returned
    /// * steps resulting in ensembles for which `energy_fn(&mut ensemble)` is `None`
    /// will always be rejected 
    pub fn init_interval_heuristik<F>(
        &mut self,
        overlap: usize,
        energy_fn: F,
        step_limit: Option<u64>,
    ) -> Result<(), WangLandauErrors>
    where F: Fn(&mut E) -> Option<T>,
        Hist: HistogramIntervalDistance<T>
    {
        let overlap = overlap.max(1);
        self.init(&energy_fn, step_limit)?;
        let mut old_dist = self.histogram
            .interval_distance_overlap(
                self.old_energy_clone(),
                overlap
            );
        
        let dist = |h: &Hist, val: T| h.interval_distance_overlap(val, overlap);
        let mut step_count = 0;
        while old_dist != 0 {
            self.greedy_helper(
                &mut old_dist,
                &energy_fn,
                dist
            );
            if let Some(limit) = step_limit {
                if limit == step_count{
                    return Err(WangLandauErrors::InitFailed);
                }
                step_count += 1;
            }
        }
        self.end_init();
        Ok(())
    }

    /// # Find a valid starting Point
    /// * if the ensemble is already at a valid starting point,
    /// the ensemble is left unchanged (as long as your energy calculation does not change the ensemble)
    /// * Uses a greedy heuristik. Performs markov steps. If that brought us closer to the target interval,
    /// the step is accepted. Otherwise it is rejected
    /// # Parameter
    /// * `step_limit`: Some(val) -> val is max number of steps tried, if no valid state is found, it will return an Error. None -> will loop until either 
    /// a valid state is found or forever
    /// * `energy_fn` function calculating `Some(energy)` of the system
    /// or rather the Parameter of which you wish to obtain the probability distribution.
    ///  Has to be the same function as used for the wang landau simulation later.
    /// If there are any states, for which the calculation is invalid, `None` should be returned
    /// * steps resulting in ensembles for which `energy_fn(&mut ensemble)` is `None`
    /// will always be rejected 
    pub fn init_greedy_heuristic<F>(
        &mut self,
        energy_fn: F,
        step_limit: Option<u64>,
    ) -> Result<(), WangLandauErrors>
    where F: Fn(&mut E) -> Option<T>,
    {
        self.init(&energy_fn, step_limit)?;
        let mut old_distance = self.histogram
            .distance(self.old_energy_clone());
        let mut step_count = 0;
        while old_distance != 0.0 {
            self.greedy_helper(
                &mut old_distance,
                &energy_fn,
                Hist::distance
            );
            if let Some(limit) = step_limit {
                if limit == step_count{
                    return Err(WangLandauErrors::InitFailed);
                }
                step_count += 1;
            }
        }
        self.end_init();
        Ok(())
    }

    /// # Wang Landau
    /// * calls `self.wang_landau_step(energy_fn)` until `self.is_converged` 
    /// or `condition(&self)` is false
    pub fn wang_landau_while<F, W>(
        &mut self,
        energy_fn: F,
        mut condition: W
    ) where F: Fn(&mut E) -> Option<T>,
        W: FnMut(&Self) -> bool,
    {
        while !self.is_converged() && condition(&self) {
            self.wang_landau_step(&energy_fn);
        }
    }

    /// # Wang Landau
    /// * calls `self.wang_landau_step(energy_fn, valid_ensemble)` until `self.is_converged` 
    pub fn wang_landau_convergence<F>(
        &mut self,
        energy_fn: F,
    )where F: Fn(&mut E) -> Option<T>,
    {
        while !self.is_converged() {
            self.wang_landau_step(&energy_fn);
        }
    }

    /// # Wang Landau Step
    /// * performs a single Wang Landau step
    /// # Parameter
    /// * `energy_fn` function calculating `Some(energy)` of the system
    /// or rather the Parameter of which you wish to obtain the probability distribution.
    /// If there are any states, for which the calculation is invalid, `None` should be returned
    /// * steps resulting in ensembles for which `energy_fn(&mut ensemble)` is `None`
    /// will always be rejected 
    /// # Important
    /// * You have to call one of the `self.init*` functions before calling this one - 
    /// **will panic otherwise**
    pub fn wang_landau_step<F>(
        &mut self,
        energy_fn: F,
    )where F: Fn(&mut E) -> Option<T>
    {
        let old_bin = self.old_bin.expect(
            "Error - self.old_bin invalid - Did you forget to call one of the `self.init*` members for initialization?"
        );
        debug_assert!(
            self.old_energy.is_some(),
            "Error - self.old_energy invalid - Did you forget to call one of the `self.init*` members for initialization?"
        );

        self.step_count += 1;
        let step_size = self.get_stepsize();


        let steps = self.ensemble.m_steps(step_size);
        
        self.check_refine();
        
        let current_energy = match energy_fn(&mut self.ensemble){
            Some(energy) => energy,
            None => {
                self.count_rejected(step_size);
                self.histogram.count_index(old_bin).unwrap();
                self.log_density[old_bin] += self.log_f;
                self.ensemble.undo_steps_quiet(steps);
                return;
            }
        };
        
        match self.histogram.get_bin_index(&current_energy)
        {
            Ok(current_bin) => {
                let accept_prob = self.metropolis_acception_prob(old_bin, current_bin);

                if self.rng.gen::<f64>() > accept_prob {
                    // reject step
                    self.count_rejected(step_size);
                    self.ensemble.undo_steps_quiet(steps);
                } else {
                    // accept step
                    self.count_accepted(step_size);
                    
                    self.old_energy = Some(current_energy);
                    self.old_bin = Some(current_bin);
                }
            },
            _  => {
                // invalid step -> reject
                self.count_rejected(step_size);
                self.ensemble.undo_steps_quiet(steps);
            }
        };
        
        self.histogram.count_index(self.old_bin.unwrap()).unwrap();
        self.log_density[self.old_bin.unwrap()] += self.log_f;
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand_pcg::Pcg64;
    use crate::rand::SeedableRng;
    #[test]
    fn wl_creation() {
        let mut rng = Pcg64::seed_from_u64(2239790);
        let ensemble: ErEnsembleC<EmptyNode, _> = ErEnsembleC::new(
            100,
            3.01,
            Pcg64::from_rng(&mut rng).unwrap()
        );
        let histogram = HistogramFast::new_inclusive(50, 100).unwrap();
        let mut wl= WangLandauAdaptive::new(
            0.00075,
            ensemble,
            Pcg64::from_rng(&mut rng).unwrap(),
            30,
            5,
            50,
            7,
            0.075,
            histogram,
            1000
        ).unwrap();

        wl.init_mixed_heuristik(
            3,
            6400i16,
            |e|  {
                e.graph().q_core(3)
            },
            None
        ).unwrap();

        wl.wang_landau_convergence(
            |e| e.graph().q_core(3)
        );
    }
}
