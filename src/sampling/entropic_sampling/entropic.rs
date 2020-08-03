use crate::{rand::{Rng, seq::*}, *};
use std::{marker::PhantomData, iter::*};
use crate::sampling::*;
use std::collections::*;

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
/// Error states, that entropic sampling, or the creation of `EntropicAdaptive`
/// could encounter
pub enum EntropicErrors {
    /// # source (`WangLandauAdaptive`) was in an invalid state
    /// * did you forget to use one of the `init*` methods to initialize a valid
    /// WangLandau state? 
    InvalidWangLandau,

    /// Still in the process of gathering statistics
    /// Not enough to make an estimate
    NotEnoughStatistics,
    /// Still Gathering Statistics, this is only an estimate!
    EstimatedStatistic(Vec<f64>),
}

/// # Entropic sampling made easy
/// > J. Lee,
/// > “New Monte Carlo algorithm: Entropic sampling,”
/// > Phys. Rev. Lett. 71, 211–214 (1993),
/// > DOI: [10.1103/PhysRevLett.71.211](https://doi.org/10.1103/PhysRevLett.71.211)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct EntropicAdaptive<Hist, R, E, S, Res, T>
{
    rng: R,
    samples_per_trial: usize,
    trial_list: Vec<usize>,
    best_of_steps: Vec<usize>,
    min_best_of_count: usize,
    best_of_threshold: f64,
    ensemble: E,
    step_marker: PhantomData::<S>,
    step_res_marker: PhantomData<Res>,
    accepted_step_hist: Vec<usize>,
    rejected_step_hist: Vec<usize>,
    total_steps_rejected: usize,
    total_steps_accepted: usize,
    min_step: usize,
    counter: usize,
    step_count: usize,
    step_goal: usize,
    histogram: Hist,
    log_density: Vec<f64>,
    old_energy: T,
    old_bin: usize,
    adjust_bestof_every: usize,
}

impl<Hist, R, E, S, Res, T> EntropicAdaptive<Hist, R, E, S, Res, T>
where Hist: Histogram,
    R: Rng
{
    /// # Creatses EntropicAdaptive from a `WangLandauAdaptive` state
    /// * `WangLandauAdaptive` state needs to be valid, i.e., you must have called one of the `init*` methods
    /// - this ensures, that the members `old_energy` and `old_bin` are not `None`
    pub fn from_wl(mut wl: WangLandauAdaptive<Hist, R, E, S, Res, T>) -> Result<Self, EntropicErrors>
    {
        if wl.old_bin.is_none() || wl.old_energy.is_none() {
            return Err(EntropicErrors::InvalidWangLandau);
        }
        wl.accepted_step_hist
            .iter_mut()
            .for_each(|v| *v = 0);
        wl.rejected_step_hist
            .iter_mut()
            .for_each(|v| *v = 0);
        wl.best_of_steps.clear();
        wl.histogram.reset();
        wl.trial_list.shuffle(&mut wl.rng);
        
        Ok(
            Self{
                counter: 0,
                step_marker: wl.step_marker,
                step_res_marker: wl.step_res_marker,
                log_density: wl.log_density,
                old_energy: wl.old_energy.unwrap(),
                old_bin: wl.old_bin.unwrap(),
                accepted_step_hist: wl.accepted_step_hist,
                rejected_step_hist: wl.rejected_step_hist,
                total_steps_accepted: 0,
                total_steps_rejected: 0,
                min_step: wl.min_step,
                min_best_of_count: wl.min_best_of_count,
                best_of_steps: wl.best_of_steps,
                best_of_threshold: wl.best_of_threshold,
                samples_per_trial: wl.samples_per_trial,
                rng: wl.rng,
                trial_list: wl.trial_list,
                ensemble: wl.ensemble,
                step_count: 0,
                step_goal: wl.step_count,
                histogram: wl.histogram,
                adjust_bestof_every: 10usize.max(4 * wl.check_refine_every),
            }   
        )
    }

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

    /// # Number of entropic steps done until now
    /// * will be reset by [`self.refine_estimate`](#method.refine_estimate)
    #[inline]
    pub fn step_count(&self) -> usize
    {
        self.step_count
    }

    /// # Number of entropic steps to be performed
    /// * if `self` was created from `WangLandauAdaptive`,
    /// `step_goal` will be equal to the number of WangLandau steps, that were performed
    #[inline]
    pub fn step_goal(&self) -> usize
    {
        self.step_goal
    }

    /// # Number of entropic steps to be performed
    /// * set the number of steps to be performed by entropic sampling
    #[inline]
    pub fn set_step_goal(&mut self, step_goal: usize){
        self.step_goal = step_goal;
    }

    /// # Smallest possible markov step (`m_steps` of MarkovChain trait) by entropic step
    #[inline]
    pub fn min_step_size(&self) -> usize
    {
        self.min_step
    }

    /// # Largest possible markov step (`m_steps` of MarkovChain trait) by entropic step
    #[inline]
    pub fn max_step_size(&self) -> usize 
    {
        self.min_step + self.accepted_step_hist.len() - 1
    }

    /// # Currently used best of
    /// * might have length 0, if statistics are still being gathered
    /// * otherwise this contains the step sizes, from which the next stepsize
    /// is drawn uniformly
    #[inline]
    pub fn best_of_steps(&self) -> &Vec<usize>
    {
        &self.best_of_steps
    }

    /// # Fraction of steps accepted since the statistics were reset the last time
    /// * (steps accepted since last reset) / (steps since last reset)
    /// * `NaN` if no steps were performed yet
    pub fn fraction_accepted_current(&self) -> f64 {
        let accepted: usize = self.accepted_step_hist.iter().sum();
        let total = accepted + self.rejected_step_hist.iter().sum::<usize>();
        if total == 0 {
            f64::NAN
        } else {
            accepted as f64 / total as f64
        }
    }

    /// # Fraction of steps accepted since the creation of `self`
    /// * total_steps_accepted / total_steps
    /// * `NaN` if no steps were performed yet
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

    /// * returns the (non normalized) log_density estimate log(P(E)), with which the simulation was started
    /// * if you created this from a WangLandau simulation, this is the result of the WangLandau Simulation
    pub fn log_density_estimate(&self) -> &Vec<f64>
    {
        &self.log_density
    }

    /// # Return current state of histogram
    pub fn hist(&self) -> &Hist
    {
        &self.histogram
    }

    /// calculates the (non normalized) log_density estimate log(P(E)) according to the [paper](#entropic-sampling-made-easy)
    pub fn log_density_refined(&self) -> Vec<f64> {
        let mut log_density = Vec::with_capacity(self.log_density.len());
        log_density.extend(
            self.log_density
                .iter()
                .zip(self.histogram.hist().iter())
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
        self.counter = 0;
        self.step_count = 0;
        self.best_of_steps.clear();
        self.histogram.reset();
        self.trial_list.shuffle(&mut self.rng);

        self.total_steps_accepted += self.accepted_step_hist.iter().sum::<usize>();
        self.accepted_step_hist
            .iter_mut()
            .for_each(|entry| *entry = 0);

        self.total_steps_rejected += self.rejected_step_hist.iter().sum::<usize>();
        self.rejected_step_hist
            .iter_mut()
            .for_each(|entry| *entry = 0);

        estimate
    }

    /// # How often to adjust `bestof_steps`?
    /// * if you try to set a value smaller 10, it will be set to 10
    /// * will reevalute the statistics every `adjust_bestof_every` steps,
    /// - this will not start new statistics gathering but just trigger a reevaluation of
    /// the gathered statistics (should be O(max_stepsize - min_stepsize))
    #[inline]
    pub fn set_adjust_bestof_every(&mut self, adjust_bestof_every: usize)
    {
        self.adjust_bestof_every = adjust_bestof_every.max(10);
    }

    /// Is the simulation in the process of rebuilding the statistics,
    /// i.e., is it currently trying many differnt step sizes?
    #[inline]
    pub fn is_rebuilding_statistics(&self) -> bool
    {
        self.counter < self.trial_list.len()
    }

    fn statistic_bin_not_hit(&self) -> bool
    {
        self.accepted_step_hist
            .iter()
            .zip(self.rejected_step_hist.iter())
            .any(|(a, r )| a + r == 0)
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

    fn adjust_bestof(&mut self){
        self.best_of_steps.clear();
        self.generate_bestof();
    }

    fn get_stepsize(&mut self) -> usize {
        match self.trial_list.get(self.counter) {
            None => {
                if self.best_of_steps.is_empty(){
                    self.generate_bestof();
                }
                else if self.counter % self.adjust_bestof_every == 0 {
                    self.adjust_bestof();
                }
                *self.best_of_steps.choose(&mut self.rng).unwrap()
            },
            Some(&step_size) => {
                step_size
            },
        }
    }

    #[inline]
    fn count_accepted(&mut self, size: usize){
        self.accepted_step_hist[size - self.min_step] += 1;
        self.counter += 1;
    }

    #[inline]
    fn count_rejected(&mut self, size: usize){
        self.rejected_step_hist[size - self.min_step] += 1;
        self.counter += 1;
    }

    /// **panics** if index is invalid
    #[inline]
    fn metropolis_acception_prob(&self, new_bin: usize) -> f64
    {
        1.0f64.min(
            (self.log_density[self.old_bin] - self.log_density[new_bin])
                .exp()
        )
    }
}

impl<Hist, R, E, S, Res, T> EntropicAdaptive<Hist, R, E, S, Res, T>
where Hist: Histogram + HistogramVal<T>,
    R: Rng,
    E: MarkovChain<S, Res>,
    T: Clone,
{
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
        let step_size = self.get_stepsize();


        let steps = self.ensemble.m_steps(step_size);
        
        let current_energy = match energy_fn(&mut self.ensemble){
            Some(energy) => energy,
            None => {
                self.count_rejected(step_size);
                self.histogram.count_index(self.old_bin).unwrap();
                self.ensemble.undo_steps_quiet(steps);
                return;
            }
        };
        
        match self.histogram.get_bin_index(&current_energy)
        {
            Ok(current_bin) => {
                let accept_prob = self.metropolis_acception_prob(current_bin);

                if self.rng.gen::<f64>() > accept_prob {
                    // reject step
                    self.count_rejected(step_size);
                    self.ensemble.undo_steps_quiet(steps);
                } else {
                    // reject step
                    self.count_accepted(step_size);
                    
                    self.old_energy = current_energy;
                    self.old_bin = current_bin;
                }
            },
            _  => {
                // invalid step -> reject
                self.count_rejected(step_size);
                self.ensemble.undo_steps_quiet(steps);
            }
        };
        
        self.histogram.count_index(self.old_bin).unwrap();
    }
}
