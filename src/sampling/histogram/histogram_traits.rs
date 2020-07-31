use std::borrow::*;

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

/// # Implements histogram
/// * anything that implements `Histogram` should also implement the trait `HistogramVal`
pub trait Histogram {
    /// # `self.hist[index] += 1`, `Err()` if `index` out of bounds
    fn count_index(&mut self, index: usize) -> Result<(), HistErrors>;
    /// # the created histogram
    fn hist(&self) -> &Vec<usize>;
    /// # How many bins the histogram contains
    fn bin_count(&self) -> usize
    {
        self.hist().len()
    }
    /// reset the histogram to zero
    fn reset(&mut self);

    /// check if any bin was not hit yet
    fn any_bin_zero(&self) -> bool
    {
        self.hist()
            .iter()
            .any(|&val| val == 0)
    }
}

/// * trait used for mapping values of arbitrary type `T` to bins
/// * used to create a histogram
pub trait HistogramVal<T>: Histogram{
    /// convert val to the respective histogram index
    fn get_bin_index<V: Borrow<T>>(&self, val: V) -> Result<usize, HistErrors>;
    /// count val. Some(index), if inside of hist, None if val is invalid
    fn count_val<V: Borrow<T>>(&mut self, val: V) -> Result<usize, HistErrors>
    {
        let id = self.get_bin_index(val)?;
        self.count_index(id)
            .map(|_| id)
    }
    /// # binning borders
    /// * the borders used to bin the values
    /// * any val which fullfills `self.border[i] <= val < self.border[i + 1]` 
    /// will get index `i`.
    /// * **Note** that the last border is exclusive
    fn borders_clone(&self) -> Result<Vec<T>, HistErrors>;
    /// does a value correspond to a valid bin?
    fn is_inside<V: Borrow<T>>(&self, val: V) -> bool;
    /// opposite of `is_inside`
    fn not_inside<V: Borrow<T>>(&self, val: V) -> bool;
    /// get the left most border (inclusive)
    fn get_left(&self) -> T;
    /// get the right most border (exclusive)
    fn get_right(&self) -> T;
    /// # calculates some sort of absolute distance to the nearest valid bin
    /// * any invalid numbers (like NAN or INFINITY) should have the highest distance possible
    /// * if a value corresponds to a valid bin, the distance should be zero
    fn distance(&self, val: T) -> f64;
}

/// Distance metric for how far a value is from a valid interval
pub trait HistogramIntervalDistance<T> {
    /// # Distance metric for how far a value is from a valid interval
    /// * partitions in more intervals, checks which bin interval a bin corresponds to 
    /// and returns distance of said interval to the target interval
    /// * used for heuristiks
    /// * overlap has to be bigger 0
    fn interval_distance_overlap(&self, val: T, overlap: usize) -> usize;
}


/// Possible Errors of the traits `Histogram` and `HistogramVal`
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub enum HistErrors{
    /// A histogram without any bins does not make sense!
    NoBins,

    /// Nothing can hit the bin! (left >= right?)
    IntervalWidthZero,

    /// Invalid value
    OutsideHist,

    /// Underflow occured
    Underflow,

    /// Overflow occured,
    Overflow,

    /// Error while casting to usize
    UsizeCastError,
    
    /// Something went wrong wile casting!
    CastError,

    /// Could be NAN, INFINITY or similar
    InvalidVal,

    /// Cannot create requested interval with 
    /// bins, that all have the same width!
    ModuloError
}