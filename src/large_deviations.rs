//! # Helper for large deviation methods
//!

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};


/// For saving MetropolisState + corresponding ensemble in one file
#[derive(Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct MetropolisSave<E, R> {
    state: MetropolisState<R>,
    ensemble: E,
}

impl<E, R> MetropolisSave<E, R> {
    /// create a new save state for a Metropolis Monte Carlo Simulation
    pub fn new(ensemble: E, state: MetropolisState<R>) -> Self
    {
        Self{
            ensemble,
            state,
        }
    }

    /// Convert `self` back into the `ensemble` and the `MetropolisState`
    pub fn unpack(self) -> (E, MetropolisState<R>) {
        (self.ensemble, self.state)
    }
}

/// used to store the current state of the monte carlo simulation
#[derive(Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct MetropolisState<R>
{
    stepsize: usize,
    step_target: usize,
    m_beta: f64,
    current_energy: f64,
    counter: usize,
    rng: R,
}

impl<R> MetropolisState<R> {
    pub(crate) fn new(
        stepsize: usize,
        step_target: usize,
        m_beta: f64,
        rng: R,
        current_energy: f64,
        counter: usize
    ) -> Self {
        assert!(
            stepsize > 0,
            "StepSize 0 is not allowed!"
        );
        Self{
            m_beta,
            rng,
            current_energy,
            counter,
            step_target,
            stepsize
        }
    }

    /// returns stored `m_beta` value (-&beta; for metropolis)
    pub fn m_beta(&self) -> f64 {
        self.m_beta
    }

    /// returns stored value for `current_energy`
    pub fn current_energy(&self) -> f64 {
        self.current_energy
    }

    /// returns stored value for the `counter`, i.e., where to resume iteration
    pub fn counter(&self) -> usize {
        self.counter
    }

    /// return current `stepsize`
    pub fn stepsize(&self) -> usize {
        self.stepsize
    }

    /// * change the `stepsize`
    /// * **panics** if you try to set the stepsize to 0
    pub fn set_stepsize(&mut self, stepsize: usize) {
        assert!(
            stepsize > 0,
            "StepSize 0 is not allowed!"
        );
        self.stepsize = stepsize;
    }

    /// returns, how many steps should be performed in total
    pub fn step_target(&self) -> usize {
        self.step_target
    }

    /// converts `self` in underlying rng generator
    pub fn to_rng(self) -> R {
        self.rng
    }

    /// * trys to increase the step target.
    /// * succeeds (returns `true`) if `self.step_target() <= new_target`, else it fails
    /// * returns `false` if it fails
    pub fn increase_step_target(&mut self, new_target: usize) -> bool {
        if self.step_target <= new_target {
            self.step_target = new_target;
            true
        } else {
            false
        }
    }
}
