use crate::{rand::Rng, *};
use std::{marker::PhantomData, iter::*};
use crate::sampling::*;
use std::convert::*;
use std::io::Write;

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};


/// # Entropic sampling made easy
/// > J. Lee,
/// > “New Monte Carlo algorithm: Entropic sampling,”
/// > Phys. Rev. Lett. 71, 211–214 (1993),
/// > DOI: [10.1103/PhysRevLett.71.211](https://doi.org/10.1103/PhysRevLett.71.211)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct EntropicSampling<Hist, R, E, S, Res, Energy>
{
    rng: R,
    ensemble: E,
    step_marker: PhantomData::<S>,
    step_res_marker: PhantomData<Res>,
    total_steps_rejected: usize,
    total_steps_accepted: usize,
    step_size: usize,
    step_count: usize,
    step_goal: usize,
    hist: Hist,
    log_density: Vec<f64>,
    old_energy: Energy,
    old_bin: usize,
}

impl<Hist, R, E, S, Res, Energy> TryFrom<WangLandau1T<Hist, R, E, S, Res, Energy>>
    for EntropicSampling<Hist, R, E, S, Res, Energy>
    where 
        Hist: Histogram,
        R: Rng
{
    type Error = EntropicErrors;
    fn try_from(mut wl: WangLandau1T<Hist, R, E, S, Res, Energy>) -> Result<Self, Self::Error> {
        if wl.energy().is_none() {
            return Err(EntropicErrors::InvalidWangLandau);
        }
        
        wl.hist.reset();
        
        Ok(
            Self{
                step_goal: wl.step_counter(),
                step_marker: wl.marker1,
                step_res_marker: wl.marker2,
                log_density: wl.log_density,
                old_energy: wl.old_energy.unwrap(),
                old_bin: wl.old_bin,
                total_steps_accepted: 0,
                total_steps_rejected: 0,
                rng: wl.rng,
                ensemble: wl.ensemble,
                step_count: 0,
                hist: wl.hist,
                step_size: wl.step_size,
            }   
        )
    }
}

impl<Hist, R, E, S, Res, T> TryFrom<WangLandauAdaptive<Hist, R, E, S, Res, T>>
    for EntropicSampling<Hist, R, E, S, Res, T>
    where 
        Hist: Histogram,
        R: Rng
{
    type Error = EntropicErrors;
    /// Uses as stepsize: first entry of bestof. If bestof is empty, it uses 
    /// `wl.min_step_size() + (wl.max_step_size() - wl.max_step_size()) / 2 `
    fn try_from(mut wl: WangLandauAdaptive<Hist, R, E, S, Res, T>) -> Result<Self, Self::Error> {
        if wl.energy().is_none() {
            return Err(EntropicErrors::InvalidWangLandau);
        }
        
        wl.histogram.reset();

        let step_size = wl.best_of_steps
            .first()
            .cloned()
            .unwrap_or( wl.min_step_size() + (wl.max_step_size() - wl.max_step_size()) / 2 );
        
        Ok(
            Self{
                step_goal: wl.step_counter(),
                step_marker: wl.step_marker,
                step_res_marker: wl.step_res_marker,
                log_density: wl.log_density,
                old_energy: wl.old_energy.unwrap(),
                old_bin: wl.old_bin.unwrap(),
                total_steps_accepted: 0,
                total_steps_rejected: 0,
                rng: wl.rng,
                ensemble: wl.ensemble,
                step_count: 0,
                hist: wl.histogram,
                step_size,
            }   
        )
    }
}

impl<Hist, R, E, S, Res, T> EntropicSampling<Hist, R, E, S, Res, T>
{

    /// # Current state of the Ensemble
    #[inline]
    pub fn ensemble(&self) -> &E
    {
        &self.ensemble
    }

    /// # Energy of ensemble
    /// * assuming `energy_fn` (see `self.entropic_step` etc.) 
    /// is deterministic and will allways give the same result for the same ensemble,
    /// this returns the energy of the current ensemble
    #[inline]
    pub fn energy(&self) -> &T
    {
        &self.old_energy
    }

    /// # Number of entropic steps to be performed
    /// * set the number of steps to be performed by entropic sampling
    #[inline]
    pub fn set_step_goal(&mut self, step_goal: usize){
        self.step_goal = step_goal;
    }

    /// # Smallest possible markov step (`m_steps` of MarkovChain trait) by entropic step
    #[inline]
    pub fn step_size(&self) -> usize
    {
        self.step_size
    }

    /// # Fraction of steps accepted since the creation of `self`
    /// * total_steps_accepted / total_steps
    /// * `NaN` if no steps were performed yet
    pub fn fraction_accepted_total(&self) -> f64 {
        let total = self.total_steps_accepted + self.total_steps_rejected;
        if total == 0 {
            f64::NAN
        } else {
            self.total_steps_accepted as f64 / total as f64
        }
    }

    /// * returns the (non normalized) log_density estimate log(P(E)), with which the simulation was started
    /// * if you created this from a WangLandau simulation, this is the result of the WangLandau Simulation
    pub fn log_density_estimate(&self) -> &Vec<f64>
    {
        &self.log_density
    }

    /// # Return current state of histogram
    pub fn hist(&self) -> &Hist
    {
        &self.hist
    }
}
impl<Hist, R, E, S, Res, T> EntropicSampling<Hist, R, E, S, Res, T>
where Hist: Histogram,
    R: Rng
{

    /// # Creates Entropic from a `WangLandauAdaptive` state
    /// * `WangLandauAdaptive` state needs to be valid, i.e., you must have called one of the `init*` methods
    /// - this ensures, that the members `old_energy` and `old_bin` are not `None`
    pub fn from_wl(wl: WangLandau1T<Hist, R, E, S, Res, T>) -> Result<Self, EntropicErrors>
    {
        wl.try_into()
    }

        /// # Creates Entropic from a `WangLandauAdaptive` state
    /// * `WangLandauAdaptive` state needs to be valid, i.e., you must have called one of the `init*` methods
    /// - this ensures, that the members `old_energy` and `old_bin` are not `None`
    pub fn from_wl_adaptive(wl: WangLandauAdaptive<Hist, R, E, S, Res, T>) -> Result<Self, EntropicErrors>
    {
        wl.try_into()
    }

    /// calculates the (non normalized) log_density estimate log(P(E)) according to the [paper](#entropic-sampling-made-easy)
    pub fn log_density_refined(&self) -> Vec<f64> {
        let mut log_density = Vec::with_capacity(self.log_density.len());
        log_density.extend(
            self.log_density
                .iter()
                .zip(self.hist.hist().iter())
                .map(
                    |(&log_p, &h)|
                    {
                        if h == 0 {
                            log_p
                        } else {
                            log_p + (h as f64).ln()
                        }
                    }
                )
        );
        log_density
    }


    /// # Calculates `self.log_density_refined` and uses that as estimate for a the entropic sampling simulation
    /// * returns old estimate
    /// # prepares `self` for a new entropic simulation
    /// * sets new estimate for log(P(E))
    /// * resets statistic gathering
    /// * resets step_count
    pub fn refine_estimate(&mut self) -> Vec<f64>
    {
        let mut estimate = self.log_density_refined();
        std::mem::swap(&mut estimate, &mut self.log_density);
        
        self.step_count = 0;

        self.hist.reset();

        self.total_steps_accepted = 0;

        self.total_steps_rejected = 0;

        estimate
    }


    #[inline(always)]
    fn count_accepted(&mut self){
        self.total_steps_accepted +=1;
        
    }

    #[inline(always)]
    fn count_rejected(&mut self){
        self.total_steps_rejected += 1;
    }

    /// **panics** if index is invalid
    #[inline]
    fn metropolis_acception_prob(&self, new_bin: usize) -> f64
    {
        (self.log_density[self.old_bin] - self.log_density[new_bin])
                .exp()
    }
}

impl<Hist, R, E, S, Res, T> EntropicSampling<Hist, R, E, S, Res, T>
where Hist: Histogram + HistogramVal<T>,
    R: Rng,
    E: MarkovChain<S, Res>,
    T: Clone,
{

    /// # Entropic sampling
    /// * performs `self.entropic_step(energy_fn)` until `condition` is false
    /// * **Note**: you have access to the current step_count (`self.step_count()`)
    /// # Parameter
    /// * `energy_fn` function calculating `Some(energy)` of the system
    /// or rather the Parameter of which you wish to obtain the probability distribution.
    /// If there are any states, for which the calculation is invalid, `None` should be returned
    /// * steps resulting in ensembles for which `energy_fn(&mut ensemble)` is `None`
    /// will always be rejected 
    /// * **Important** `energy_fn`: should be the same as used for Wang Landau, otherwise the results will be wrong!
    /// * `print_fn`: see below
    /// # Correlations
    /// * if you want to measure correlations between "energy" and other measurable quantities,
    /// use `print_fn`, which will be called after each step - use this function to write to 
    /// a file or whatever you desire
    /// * Note: You do not have to recalculate the energy, if you need it in `print_fn`:
    ///  just call `self.energy()` 
    /// * you have access to your ensemble with `self.ensemble()`
    /// * if you do not need it, you can use `|_|{}` as `print_fn`
    pub fn entropic_sampling_while<F, G, W>(
        &mut self,
        energy_fn: F,
        mut print_fn: G,
        mut condition: W
    ) where F: Fn(&mut E) -> Option<T>,
        G: FnMut(&Self) -> (),
        W: FnMut(&Self) -> bool
    {
        while condition(self) {
            self.entropic_step(&energy_fn);
            print_fn(&self);
        }
    }

    /// # Entropic sampling
    /// * performs `self.entropic_step(energy_fn)` until `self.step_count == self.step_goal`
    /// # Parameter
    /// * `energy_fn` function calculating `Some(energy)` of the system
    /// or rather the Parameter of which you wish to obtain the probability distribution.
    /// If there are any states, for which the calculation is invalid, `None` should be returned
    /// * steps resulting in ensembles for which `energy_fn(&mut ensemble)` is `None`
    /// will always be rejected 
    /// * **Important** `energy_fn`: should be the same as used for Wang Landau, otherwise the results will be wrong!
    /// * `print_fn`: see below
    /// # Correlations
    /// * if you want to measure correlations between "energy" and other measurable quantities,
    /// use `print_fn`, which will be called after each step - use this function to write to 
    /// a file or whatever you desire
    /// * Note: You do not have to recalculate the energy, if you need it in `print_fn`:
    ///  just call `self.energy()` 
    /// * you have access to your ensemble with `self.ensemble()`
    /// * if you do not need it, you can use `|_|{}` as `print_fn`
    pub fn entropic_sampling<F, G>(
        &mut self,
        energy_fn: F,
        mut print_fn: G,
    ) where F: Fn(&mut E) -> Option<T>,
        G: FnMut(&Self) -> ()
    {
        while self.step_count < self.step_goal {
            self.entropic_step(&energy_fn);
            print_fn(&self);
        }
    }

    /// # Entropic step
    /// * performs a single step
    /// # Parameter
    /// * `energy_fn` function calculating `Some(energy)` of the system
    /// or rather the Parameter of which you wish to obtain the probability distribution.
    /// If there are any states, for which the calculation is invalid, `None` should be returned
    /// * steps resulting in ensembles for which `energy_fn(&mut ensemble)` is `None`
    /// will always be rejected 
    /// # Important
    /// * `energy_fn`: should be the same as used for Wang Landau, otherwise the results will be wrong!
    pub fn entropic_step<F>(
        &mut self,
        energy_fn: F,
    )where F: Fn(&mut E) -> Option<T>
    {

        self.step_count += 1;


        let steps = self.ensemble.m_steps(self.step_size);
        
        let current_energy = match energy_fn(&mut self.ensemble){
            Some(energy) => energy,
            None => {
                self.count_rejected();
                self.hist.count_index(self.old_bin).unwrap();
                self.ensemble.undo_steps_quiet(steps);
                return;
            }
        };
        
        match self.hist.get_bin_index(&current_energy)
        {
            Ok(current_bin) => {
                let accept_prob = self.metropolis_acception_prob(current_bin);

                if self.rng.gen::<f64>() > accept_prob {
                    // reject step
                    self.count_rejected();
                    self.ensemble.undo_steps_quiet(steps);
                } else {
                    // accept step
                    self.count_accepted();
                    
                    self.old_energy = current_energy;
                    self.old_bin = current_bin;
                }
            },
            _  => {
                // invalid step -> reject
                self.count_rejected();
                self.ensemble.undo_steps_quiet(steps);
            }
        };
        
        self.hist
            .count_index(self.old_bin)
            .unwrap();
    }
}


impl<Hist, R, E, S, Res, T> Entropic for EntropicSampling<Hist, R, E, S, Res, T>
where Hist: Histogram,
    R: Rng,
{
    /// # Number of entropic steps done until now
    /// * will be reset by [`self.refine_estimate`](#method.refine_estimate)
    #[inline]
    fn step_counter(&self) -> usize
    {
        self.step_count
    }

    /// # Number of entropic steps to be performed
    /// * if `self` was created from `WangLandauAdaptive`,
    /// `step_goal` will be equal to the number of WangLandau steps, that were performed
    #[inline]
    fn step_goal(&self) -> usize
    {
        self.step_goal
    }

    fn log_density(&self) -> Vec<f64> {
        self.log_density_refined()
    }

    fn write_log<W: Write>(&self, mut w: W) -> Result<(), std::io::Error> {
        writeln!(w,
            "#Acceptance prob_total: {}\n#total_steps: {:e}\n#step_goal {:e}",
            self.fraction_accepted_total(),
            self.step_counter(),
            self.step_goal()
        )
    }
}

impl<Hist, R, E, S, Res, Energy> EntropicEnergy<Energy> for EntropicSampling<Hist, R, E, S, Res, Energy>
where Hist: Histogram,
    R: Rng,
{
    /// # Energy of ensemble
    /// * assuming `energy_fn` (see `self.entropic_step` etc.) 
    /// is deterministic and will allways give the same result for the same ensemble,
    /// this returns the energy of the current ensemble
    #[inline]
    fn energy(&self) -> &Energy
    {
        &self.old_energy
    }
}

impl<Hist, R, E, S, Res, Energy> EntropicHist<Hist> for EntropicSampling<Hist, R, E, S, Res, Energy>
where Hist: Histogram,
    R: Rng,
{
    #[inline]
    fn hist(&self) -> &Hist
    {
        &self.hist
    }
}

impl<Hist, R, E, S, Res, Energy> EntropicEnsemble<E> for EntropicSampling<Hist, R, E, S, Res, Energy>
where Hist: Histogram,
    R: Rng,
{
    fn ensemble(&self) -> &E {
        &self.ensemble
    }
}