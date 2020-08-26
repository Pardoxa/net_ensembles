use crate::sampling::wang_landau::*;
use crate::sampling::Histogram;
/// # Traits for quantities that all Wang Landau simulations have
/// * see also: `WangLandauHist`
/// * this trait is for convinience, so that you do not have
/// to write all the trait bounds of `WangLandauHist`, if you are
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
    fn is_converged(&self) -> bool{
        self.log_f() <= self.log_f_threshold()
    }
    
    /// # Current (non normalized) estimate of ln(P(E))
    /// * i.e., of the natural logarithm of the 
    /// probability density function
    /// for the requested interval
    /// * this is what we are doing the simulations for
    fn log_density(&self) -> &Vec<f64>;
    
    /// # Returns current wang landau mode
    /// * see `WangLandauMode` for an explaination
    fn mode(&self) -> WangLandauMode;
    
    /// # Counter
    /// how many wang Landau steps were performed until now?
    fn step_counter(&self) -> usize;
}

pub trait WangLandauEnsemble<E> : WangLandau
{
    fn ensemble(&self) -> &E;
}

pub trait WangLandauHist<Hist> : WangLandau
{
    /// # returns current histogram
    /// * **Note**: histogram will be reset multiple times during the simulation
    /// * please refere to the [papers](struct.WangLandauAdaptive.html#adaptive-wanglandau-1t)
    fn hist(&self) -> &Hist;
}

pub trait WangLandauEnergy<Energy> : WangLandau
{
    fn energy(&self) -> Option<&Energy>;
}

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