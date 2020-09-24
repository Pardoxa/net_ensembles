use crate::sampling::wang_landau::*;
use crate::sampling::Histogram;
use std::io::Write;
/// # Traits for quantities that all Wang Landau simulations have
/// * see also: `WangLandauHist`
/// * this trait is for convinience, so that you do not have
/// to write all the trait bounds of, e.g.,  `WangLandauHist`, if you are
/// not using functuinality, that requires it
pub trait WangLandau
{
    /// get current value of log_f
    fn log_f(&self) -> f64;

    /// # returns currently set threshold for log_f
    fn log_f_threshold(&self) -> f64;
    
    /// Try to set the threshold. 
    /// * `log_f_threshold > 0.0` has to be true
    /// * `log_f_threshold` has to be finite
    fn set_log_f_threshold(&mut self, log_f_threshold: f64) -> Result<f64, WangLandauErrors>;
    
    /// # Checks wang landau threshold
    /// * `log_f <= log_f_threshold`
    #[inline(always)]
    fn is_finished(&self) -> bool
    {
        self.log_f() <= self.log_f_threshold()
    }
    
    /// # Current (non normalized) estimate of ln(P(E))
    /// * i.e., of the natural logarithm of the 
    /// probability density function
    /// for the requested interval
    /// * this is what we are doing the simulations for
    fn log_density(&self) -> &Vec<f64>;

    /// # Current (non normalized) estimate of log10(P(E))
    /// * i.e., of logarithm with base 10 of the 
    /// probability density function
    /// for the requested interval
    /// * this is what we are doing the simulations for
    fn log_density_base10(&self) -> Vec<f64>{
        self.log_density()
            .iter()
            .map(|val| val * std::f64::consts::LOG10_E)
            .collect()
    }

    /// # Current (non normalized) estimate of log_base(P(E))
    /// * i.e., of logarithm with arbitrary base of the 
    /// probability density function
    /// for the requested interval
    /// * this is what we are doing the simulations for
    fn log_density_base(&self, base: f64) -> Vec<f64>{
        let factor = std::f64::consts::E.log(base);
        self.log_density()
            .iter()
            .map(|val| val * factor)
            .collect()
    }

    /// Writes Information about the simulation to a file.
    /// E.g. How many steps were performed.
    fn write_log<W: Write>(&self, writer: W) -> Result<(), std::io::Error>;
    
    /// # Returns current wang landau mode
    /// * see `WangLandauMode` for an explaination
    fn mode(&self) -> WangLandauMode;
    
    /// # Counter
    /// * how many wang Landau steps were performed until now?
    /// * this does not include steps, that were perfored to find a inital valid ensemble
    fn step_counter(&self) -> usize;

    /// # Counter
    /// * how many wang Landau steps were performed until now?
    /// * this includes steps, that were perfored to find a inital valid ensemble
    fn steps_total(&self) -> usize{
        self.total_steps_accepted() + self.total_steps_rejected()
    }

    /// # How many steps were accepted until now?
    /// * this includes steps, that were perfored to find a inital valid ensemble
    fn total_steps_accepted(&self) -> usize;

    /// # How many steps were rejected until now?
    /// * this includes steps, that were perfored to find a inital valid ensemble
    fn total_steps_rejected(&self) -> usize;

    /// # Calculate, which fraction of steps were accepted
    /// * this includes steps, that were perfored to find a inital valid ensemble
    /// * if no steps were performed, it returns `f64::NAN`
    fn fraction_accepted_total(&self) -> f64 {
        let total_acc = self.total_steps_accepted();
        let total_steps = total_acc + self.total_steps_rejected();

        if total_steps == 0 {
            f64::NAN
        } else {
            total_acc as f64 / total_steps as f64
        }
    }

    /// # Calculate, which fraction of steps were rejected
    /// * this includes steps, that were perfored to find a inital valid ensemble
    /// * if no steps were performed, it returns `f64::NAN`
    fn fraction_rejected_total(&self) -> f64 {
        let total_rej = self.total_steps_rejected();
        let total_steps = total_rej + self.total_steps_accepted();

        if total_steps == 0 {
            f64::NAN
        } else {
            total_rej as f64 / total_steps as f64
        }
    }
}


/// # trait to request a reference to the current (state of the) ensemble 
/// * See also [WangLandauEEH](trait.WangLandauEEH.html)
pub trait WangLandauEnsemble<E> : WangLandau
{
    /// return reference to current state of ensemble
    fn ensemble(&self) -> &E;

    /// # mutable reference to current state
    /// * **unsafe** you should not make any changes, that effect the energy state of the system.
    /// otherwise the WangLandau simulations will give false results!
    unsafe fn ensemble_mut(&mut self) -> &mut E;
}

/// # trait to request the current histogram from a WangLandau simulation
/// * Note: The histogram will likely be reset multiple times during a simulation
/// * See also [WangLandauEEH](trait.WangLandauEEH.html)
pub trait WangLandauHist<Hist> : WangLandau
{
    /// # returns current histogram
    /// * **Note**: histogram will be reset multiple times during the simulation
    /// * please refere to the [papers](struct.WangLandauAdaptive.html#adaptive-wanglandau-1t)
    fn hist(&self) -> &Hist;
}

/// # trait to request the current energy from a WangLandau simulation
/// * `None` if the energy was not calculated yet
/// * See also [WangLandauEEH](trait.WangLandauEEH.html)
pub trait WangLandauEnergy<Energy> : WangLandau
{
    /// returns the last accepted `Energy` calculated
    /// `None` if no energy was calculated yet
    fn energy(&self) -> Option<&Energy>;
}

/// Helper trait, so that you have to type less
pub trait WangLandauEEH<E, Hist, Energy> 
    : WangLandauEnergy<Energy> + WangLandauEnsemble<E>
        + WangLandauHist<Hist>{}

impl<A, E, Hist, Energy> WangLandauEEH<E, Hist, Energy> for A
    where 
    A: WangLandauEnergy<Energy> 
        + WangLandauEnsemble<E>
        + WangLandauHist<Hist>{}

pub(crate) trait WangLandau1TCalc<Hist> : WangLandauHist<Hist>
where Hist: Histogram{
    #[inline(always)]
    fn log_f_1_t(&self) -> f64 
    {
        self.hist().bin_count() as f64 / self.step_counter() as f64
    }
}

impl<A, Hist> WangLandau1TCalc<Hist> for A
    where A: WangLandauHist<Hist>,
    Hist: Histogram{}