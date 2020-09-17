use std::{marker::PhantomData, io::Write};
use crate::sampling::*;
use crate::{rand::Rng, *};
use num_traits::{Bounded, ops::wrapping::*, identities::*};

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

/// # The 1/t Wang Landau approach comes from this paper
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
pub struct WangLandau1T<Hist, Rng, Ensemble, S, Res, Energy>{
    pub(crate) ensemble: Ensemble,
    pub(crate) rng: Rng,
    pub(crate) marker1: PhantomData<S>,
    pub(crate) marker2: PhantomData<Res>,
    mode: WangLandauMode,
    pub(crate) log_density: Vec<f64>,
    pub(crate) log_f: f64,
    pub(crate) log_f_threshold: f64,
    pub(crate) step_size: usize,
    step_count: usize,
    accepted_steps_total: usize,
    recected_steps_total: usize,
    accepted_steps_current: usize,
    recected_steps_current: usize,
    pub(crate) old_bin: usize,
    pub(crate) hist: Hist,
    pub(crate) old_energy: Option<Energy>,
    check_refine_every: usize,
}


impl<Hist, R, E, S, Res, Energy> WangLandau 
    for WangLandau1T<Hist, R, E, S, Res, Energy>
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

    fn write_log<W: Write>(&self, mut writer: W) -> Result<(), std::io::Error> {
        writeln!(writer,
            "#Acceptance prob_total: {}\n#Acceptance prob current: {}\n#total_steps: {}\n#log_f: {:e}\n#Current_mode {:?}",
            self.fraction_accepted_total(),
            self.fraction_accepted_current(),
            self.step_counter(),
            self.log_f(),
            self.mode
        )?;
        writeln!(
            writer,
            "#total_steps_accepted: {}\n#total_steps_rejected: {}\n#current_accepted_steps: {}\n#current_rejected_steps: {}",
            self.accepted_steps_total,
            self.recected_steps_total,
            self.accepted_steps_current,
            self.recected_steps_current
        )
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

    fn total_steps_rejected(&self) -> usize {
        self.recected_steps_total
    }

    fn total_steps_accepted(&self) -> usize {
        self.accepted_steps_total
    }
}


impl <Hist, R, E, S, Res, Energy> WangLandauEnsemble<E> 
    for WangLandau1T<Hist, R, E, S, Res, Energy>
{
    #[inline(always)]
    fn ensemble(&self) -> &E {
        &self.ensemble
    }

    #[inline(always)]
    unsafe fn ensemble_mut(&mut self) -> &mut E {
        &mut self.ensemble
    }
}

impl <Hist, R, E, S, Res, Energy> WangLandauEnergy<Energy> 
    for WangLandau1T<Hist, R, E, S, Res, Energy>
{
    #[inline(always)]
    fn energy(&self) -> Option<&Energy> {
        self.old_energy.as_ref()
    }
}

impl <Hist, R, E, S, Res, Energy> WangLandauHist<Hist> 
    for WangLandau1T<Hist, R, E, S, Res, Energy>
{
    #[inline(always)]
    fn hist(&self) -> &Hist {
        &self.hist   
    }
}

impl<Hist, R, E, S, Res, Energy> 
    WangLandau1T<Hist, R, E, S, Res, Energy>
{
    fn fraction_accepted_total(&self) -> f64
    {
        let sum = self.accepted_steps_total + self.recected_steps_total;
        self.accepted_steps_total as f64 / sum as f64
    }
    
    fn fraction_accepted_current(&self) -> f64
    {
        let total = self.accepted_steps_current + self.recected_steps_current;
        if total == 0 {
            f64::NAN
        } else {
            self.accepted_steps_current as f64 / total as f64
        }
    }
}



impl<Hist, R, E, S, Res, Energy> 
    WangLandau1T<Hist, R, E, S, Res, Energy>
where 
    R: Rng,
    E: MarkovChain<S, Res>,
    Energy: Clone,
    Hist: Histogram + HistogramVal<Energy>
{
    /// # Create a new WangLandau simulation
    /// **IMPORTANT** You have to call one of the `init*` functions, 
    /// to create a valid state, before you can start the simulation
    /// ## Parameter
    /// * `log_f_threshold`: how small should the ln(f) (see paper) become
    /// until the simulation is finished?
    /// * `ensemble`: The ensemble to explore. 
    /// Current state of ensemble will be used as inital condition for the `init*` functions
    /// * `step_size`: The markov steps will be performed with this step size, e.g., 
    /// `ensemble.m_steps(step_size)`
    /// * `histogram`: Provides the binning. You can either use one of the already implemented
    /// histograms, like `HistU32Fast`, `HistU32`, `HistF64` etc. or implement your own by 
    /// implementing the traits `Histogram + HistogramVal<Energy>` yourself
    /// * `check_refine_every`: how often to check, if every bin in the histogram was hit.
    /// Needs to be at least 1. Good values depend on the problem at hand, but if you are 
    /// unsure, you can start with a value like 1000 
    pub fn new(
        log_f_threshold: f64,
        ensemble: E,
        rng: R,
        step_size: usize,
        histogram: Hist,
        check_refine_every: usize
    )-> Result<Self, WangLandauErrors>
    {
        if !log_f_threshold.is_finite() || log_f_threshold.is_sign_negative() 
        {
            return Err(WangLandauErrors::InvalidLogFThreshold);
        }
        else if check_refine_every == 0 {
            return Err(WangLandauErrors::CheckRefineEvery0)
        }
        let log_density = vec![0.0; histogram.bin_count()];

        Ok(
            Self{
                ensemble,
                step_count: 0,
                step_size,
                hist: histogram,
                rng,
                marker1: PhantomData::<S>,
                marker2: PhantomData::<Res>,
                log_f: 1.0,
                log_density,
                log_f_threshold,
                mode: WangLandauMode::RefineOriginal,
                recected_steps_current: 0,
                recected_steps_total: 0,
                accepted_steps_current: 0,
                accepted_steps_total: 0,
                old_bin: usize::MAX,
                old_energy: None,
                check_refine_every,
            }
        )
    }

    fn init<F>(
        &mut self,
        energy_fn: F,
        step_limit: Option<u64>
    ) -> Result<(), WangLandauErrors>
        where F: Fn(&mut E) -> Option<Energy>
    {
        self.old_energy = energy_fn(&mut self.ensemble);
        if self.old_energy.is_some(){
            return Ok(());
        }

        match step_limit {
            None => {
                loop {
                    self.ensemble.m_steps_quiet(self.step_size);
                    self.old_energy = energy_fn(&mut self.ensemble);
        
                    if self.old_energy.is_some(){
                        self.count_accepted();
                        return Ok(());
                    }
                    self.count_rejected();
                }
            },
            Some(limit) => {
                for _ in 0..limit {
                    self.ensemble.m_steps_quiet(self.step_size);
                    self.old_energy = energy_fn(&mut self.ensemble);
        
                    if self.old_energy.is_some(){
                        self.count_accepted();
                        return Ok(());
                    }
                    self.count_rejected();
                }
                Err(WangLandauErrors::InitFailed)
            }
        }
    }

    fn greedy_helper<F, H, J>(
        &mut self,
        old_distance: &mut J,
        energy_fn: F,
        distance_fn: H
    )   where F: Fn(&mut E) -> Option<Energy> + Copy,
            H: Fn(&Hist, Energy) -> J,
            J: PartialOrd
    {
        let steps = self.ensemble.m_steps(self.step_size);

        
        if let Some(energy) = energy_fn(&mut self.ensemble) {
            let distance = distance_fn(&self.hist, energy.clone());
            if distance <= *old_distance {
                self.old_energy = Some(energy);
                *old_distance = distance;
                self.count_accepted();
                return;
            }
        }

        self.count_rejected();
        self.ensemble.undo_steps_quiet(steps);
        
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
    where F: Fn(&mut E) -> Option<Energy>,
    {
        self.init(&energy_fn, step_limit)?;
        let mut old_distance = self.hist
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
    where F: Fn(&mut E) -> Option<Energy>,
        Hist: HistogramIntervalDistance<Energy>
    {
        let overlap = overlap.max(1);
        self.init(&energy_fn, step_limit)?;
        let mut old_dist = self.hist
            .interval_distance_overlap(
                self.old_energy_clone(),
                overlap
            );
        
        let dist = |h: &Hist, val: Energy| h.interval_distance_overlap(val, overlap);
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
    where F: Fn(&mut E) -> Option<Energy>,
        Hist: HistogramIntervalDistance<Energy>,
        U: One + Bounded + WrappingAdd + Eq + PartialOrd
    {
        let overlap = overlap.max(1);
        self.init(&energy_fn, step_limit)?;
        if self.hist
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
        let dist_interval = |h: &Hist, val: Energy| h.interval_distance_overlap(val, overlap);
        let mut step_count = 0;
        loop {
            let current_energy = self.old_energy_clone();
            if counter == min_val {
                old_dist = self.hist.distance(current_energy);
            }else if counter == mid {
                old_dist_interval = dist_interval(&self.hist, current_energy);
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

    fn end_init(&mut self)
    {
        self.old_bin = self.hist
            .get_bin_index( 
                self.old_energy
                    .as_ref()
                    .unwrap()
            ).expect("Error in heuristic - old bin invalid");
    }

    fn old_energy_clone(&self) -> Energy {
        self.old_energy
            .as_ref()
            .unwrap()
            .clone()
    }


    fn count_accepted(&mut self){
        self.accepted_steps_current = 
            self.accepted_steps_current.saturating_add(1);
        self.accepted_steps_total = 
            self.accepted_steps_total.saturating_add(1);
    }

    fn count_rejected(&mut self){
        self.recected_steps_current = 
            self.recected_steps_current.saturating_add(1);
        self.recected_steps_total = 
            self.recected_steps_total.saturating_add(1);
    }


    fn check_refine(&mut self)
    {
        match self.mode{
            WangLandauMode::Refine1T => {
                self.log_f = self.log_f_1_t();
            },
            WangLandauMode::RefineOriginal => {
                if self.step_count % self.check_refine_every == 0 
                    && !self.hist.any_bin_zero() 
                {
                    self.recected_steps_current = 0;
                    self.accepted_steps_current = 0;
                    let ref_1_t = self.log_f_1_t();
                    self.log_f *= 0.5;
                    if self.log_f < ref_1_t {
                        self.log_f = ref_1_t;
                        self.mode = WangLandauMode::Refine1T;
                    }
                    self.hist.reset();
                }
            }
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
    )where F: Fn(&mut E) -> Option<Energy>
    {
        debug_assert!(
            self.old_energy.is_some(),
            "Error - self.old_energy invalid - Did you forget to call one of the `self.init*` members for initialization?"
        );

        self.step_count += 1;


        let steps = self.ensemble.m_steps(self.step_size);
        
        self.check_refine();
        
        let current_energy = match energy_fn(&mut self.ensemble){
            Some(energy) => energy,
            None => {
                self.count_rejected();
                self.hist.count_index(self.old_bin).unwrap();
                self.log_density[self.old_bin] += self.log_f;
                self.ensemble.undo_steps_quiet(steps);
                return;
            }
        };
        
        match self.hist.get_bin_index(&current_energy)
        {
            Ok(current_bin) => {
                let accept_prob = self.metropolis_acception_prob( current_bin);

                if self.rng.gen::<f64>() > accept_prob {
                    // reject step
                    self.count_rejected();
                    self.ensemble.undo_steps_quiet(steps);
                } else {
                    // accept step
                    self.count_accepted();
                    
                    self.old_energy = Some(current_energy);
                    self.old_bin = current_bin;
                }
            },
            _  => {
                // invalid step -> reject
                self.count_rejected();
                self.ensemble.undo_steps_quiet(steps);
            }
        };
        
        self.hist.count_index(self.old_bin).unwrap();
        self.log_density[self.old_bin] += self.log_f;
    }

    /// # Wang Landau
    /// * calls `self.wang_landau_step(energy_fn, valid_ensemble)` until `self.is_finished()` 
    pub fn wang_landau_convergence<F>(
        &mut self,
        energy_fn: F,
    )where F: Fn(&mut E) -> Option<Energy>,
    {
        while !self.is_finished() {
            self.wang_landau_step(&energy_fn);
        }
    }

    /// # Wang Landau
    /// * calls `self.wang_landau_step(energy_fn)` until `self.is_finished()` 
    /// or `condition(&self)` is false
    pub fn wang_landau_while<F, W>(
        &mut self,
        energy_fn: F,
        mut condition: W
    ) where F: Fn(&mut E) -> Option<Energy>,
        W: FnMut(&Self) -> bool,
    {
        while !self.is_finished() && condition(&self) {
            self.wang_landau_step(&energy_fn);
        }
    }


    /// **panics** if index is invalid
    fn metropolis_acception_prob(&self, new_bin: usize) -> f64
    {
        
        (self.log_density[self.old_bin] - self.log_density[new_bin])
            .exp()
        
    }
}