use num_traits::{ops::{checked::*, wrapping::*}, cast::*, identities::*, Bounded};
use crate::sampling::histogram::*;
use std::{borrow::*, ops::*};


#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};


/// # Generic Histogram for integer types
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct HistogramInt<T>
{
    bin_borders: Vec<T>,
    hist: Vec<usize>,
}

impl<T> HistogramInt<T>{
    /// similar to `self.borders_clone` but does not allocate memory
    pub fn borders(&self) -> &Vec<T>
    {
        &self.bin_borders
    }
}

impl<T> HistogramInt<T> 
where T: PartialOrd + ToPrimitive + FromPrimitive + CheckedAdd + One + HasUnsignedVersion + Bounded
        + Sub<T, Output=T> + Mul<T, Output=T> + Zero + Copy + std::fmt::Debug,
    std::ops::RangeInclusive<T>: Iterator<Item=T>,
    T::Unsigned: Bounded + HasUnsignedVersion<LeBytes=T::LeBytes, Unsigned=T::Unsigned> 
        + WrappingAdd + ToPrimitive + Sub<Output=T::Unsigned>
        + std::ops::Rem<Output=T::Unsigned> + FromPrimitive + Zero
        + std::cmp::Eq + std::ops::Div<Output=T::Unsigned>
        + Ord + std::ops::Mul<Output=T::Unsigned> + WrappingSub + Copy + std::fmt::Debug,
    std::ops::RangeInclusive<T::Unsigned>: Iterator<Item=T::Unsigned>
{
    /// # Create a new histogram
    /// * `right`: exclusive border
    /// * `left`: inclusive border
    /// * `bins`: how many bins do you need?
    /// # Note
    /// * `(right - left) % bins == 0` has to be true, otherwise
    /// the bins cannot all have the same length!
    pub fn new(left: T, right: T, bins: usize) -> Result<Self, HistErrors> {
        if left >= right {
            return Err(HistErrors::IntervalWidthZero);
        } else if bins == 0 {
            return Err(HistErrors::NoBins);
        }
        let left_u = to_u(left);
        let right_u = to_u(right);
        let border_difference = right_u - left_u;
        let b = match T::Unsigned::from_usize(bins)
        {
            Some(val) => val,
            None => return Err(HistErrors::IntervalWidthZero),
        };
        if border_difference % b != T::Unsigned::zero() {
            return Err(HistErrors::ModuloError);
        }

        let bin_size = border_difference / b;

        if bin_size <= T::Unsigned::zero() {
            return Err(HistErrors::IntervalWidthZero);
        }
        
        let hist = vec![0; bins];
        let bin_borders: Vec<_> = (T::Unsigned::zero()..=b)
            .map(|val| {
                from_u(
                    left_u + to_u(val) * bin_size
                )
            })
            .collect();
        Ok(
            Self{
                bin_borders,
                hist
            }
        )
    }
    /// # Create a new histogram
    /// * equivalent to [`Self::new(left, right + 1, bins)`](#method.new)
    /// (except that this method checks for possible overflow)
    /// # Note:
    /// * Due to implementation details, `right` cannot be `T::MAX` - 
    /// if you try, you will get `Err(HistErrors::Overflow)`
    pub fn new_inclusive(left: T, right: T, bins: usize) -> Result<Self, HistErrors>
    {
        let right = match right.checked_add(&T::one()){
            None => return Err(HistErrors::Overflow),
            Some(val) => val,
        };
        Self::new(left, right, bins)
    }
}

impl<T> Histogram for HistogramInt<T>
{
    #[inline]
    fn bin_count(&self) -> usize {
        self.hist.len()
    }

    #[inline]
    fn hist(&self) -> &Vec<usize> {
        &self.hist
    }

    #[inline]
    fn count_index(&mut self, index: usize) -> Result<(), HistErrors> {
        if index < self.hist.len()
        {
            self.hist[index] += 1;
            Ok(())
        } else {
            Err(HistErrors::OutsideHist)
        }
    }

    #[inline]
    fn reset(&mut self) {
        // compiles down to memset :)
        self.hist
            .iter_mut()
            .for_each(|h| *h = 0);
    }
}

impl<T> HistogramVal<T> for HistogramInt<T>
where T: Ord + Sub<T, Output=T> + Add<T, Output=T> + One + NumCast + Copy
{

    fn distance(&self, val: T) -> f64 {
        if self.not_inside(val) {
            let dist = if val < self.get_left() {
                self.get_left() - val
            } else {
                val - self.get_right() + T::one()
            };
            dist.to_f64().unwrap()
        } else {
            0.0
        }
    }

    #[inline]
    fn get_left(&self) -> T {
        self.bin_borders[0]
    }

    #[inline]
    fn get_right(&self) -> T {
        self.bin_borders[self.bin_borders.len() - 1]
    }

    #[inline]
    fn is_inside<V: Borrow<T>>(&self, val: V) -> bool {
        let val = *val.borrow();
        val >= self.get_left()
            && val < self.get_right()
    }

    #[inline]
    fn not_inside<V: Borrow<T>>(&self, val: V) -> bool {
        let val = *val.borrow();
        val < self.get_left()
            || val >= self.get_right()
    }

    /// None if not inside Hist covered zone
    fn get_bin_index<V: Borrow<T>>(&self, val: V) -> Result<usize, HistErrors>
    {
        let val = val.borrow();
        if self.not_inside(val)
        {
            return Err(HistErrors::OutsideHist);
        }

        self.bin_borders
            .binary_search(val.borrow())
            .or_else(|index_m1| Ok(index_m1 + 1))
    }

    fn borders_clone(&self) -> Result<Vec<T>, HistErrors> {
        Ok(self.bin_borders.clone())
    }
}

impl<T> HistogramIntervalDistance<T> for HistogramInt<T> 
where T: Ord + Sub<T, Output=T> + Add<T, Output=T> + One + NumCast + Copy
{
    fn interval_distance_overlap(&self, val: T, mut overlap: usize) -> usize {
        overlap = overlap.max(1);
        if self.not_inside(val) {
            let num_bins_overlap = 1usize.max(self.bin_count() / overlap);
            let dist = 
            if val < self.get_left() { 
                self.get_left() - val
            } else {
                val - self.get_right()
            };
            1 + dist.to_usize().unwrap() / num_bins_overlap
        } else {
            0
        }
    }
}


/// # Histogram for binning `usize` - alias for `HistogramInt<usize>`
/// * you should use `HistUsizeFast` instead, if your bins are `[left, left+1,..., right]`
pub type HistUsize = HistogramInt<usize>;
/// # Histogram for binning `u128` - alias for `HistogramInt<u128>`
/// * you should use `HistU128Fast` instead, if your bins are `[left, left+1,..., right]`
pub type HistU128 = HistogramInt<u128>;
/// # Histogram for binning `u64` - alias for `HistogramInt<u64>`
/// * you should use `HistU64Fast` instead, if your bins are `[left, left+1,..., right]`
pub type HistU64 = HistogramInt<u64>;
/// # Histogram for binning `u32` - alias for `HistogramInt<u32>`
/// * you should use `HistU32Fast` instead, if your bins are `[left, left+1,..., right]`
pub type HistU32 = HistogramInt<u32>;
/// # Histogram for binning `u16` - alias for `HistogramInt<u16>`
/// * you should use `HistU16Fast` instead, if your bins are `[left, left+1,..., right]`
pub type HistU16 = HistogramInt<u16>;
/// # Histogram for binning `u8` - alias for `HistogramInt<u8>`
/// * you should use `HistU8Fast` instead, if your bins are `[left, left+1,..., right]`
pub type HistU8 = HistogramInt<u8>;

/// # Histogram for binning `isize` - alias for `HistogramInt<isize>`
/// * you should use `HistIsizeFast` instead, if your bins are `[left, left+1,..., right]`
pub type HistIsize = HistogramInt<isize>;
/// # Histogram for binning `i128` - alias for `HistogramInt<i128>`
/// * you should use `HistI128Fast` instead, if your bins are `[left, left+1,..., right]`
pub type HistI128 = HistogramInt<i128>;
/// # Histogram for binning `i64` - alias for `HistogramInt<i64>`
/// * you should use `HistI64Fast` instead, if your bins are `[left, left+1,..., right]`
pub type HistI64 = HistogramInt<i64>;
/// # Histogram for binning `i32` - alias for `HistogramInt<i32>`
/// * you should use `HistI32Fast` instead, if your bins are `[left, left+1,..., right]`
pub type HistI32 = HistogramInt<i32>;
/// # Histogram for binning `i16` - alias for `HistogramInt<i16>`
/// * you should use `HistI16Fast` instead, if your bins are `[left, left+1,..., right]`
pub type HistI16 = HistogramInt<i16>;
/// # Histogram for binning `i8` - alias for `HistogramIntiu8>`
/// * you should use `HistI8Fast` instead, if your bins are `[left, left+1,..., right]`
pub type HistI8 = HistogramInt<i8>;

#[cfg(test)]
mod tests{
    use super::*;
    use num_traits::Bounded;
    fn hist_test_normal<T>(left: T, right: T)
    where T: num_traits::Bounded + PartialOrd + CheckedSub 
        + CheckedAdd + Zero + Ord + HasUnsignedVersion
        + One + NumCast + Copy + FromPrimitive + Bounded + std::fmt::Debug,
    std::ops::RangeInclusive<T>: Iterator<Item=T>,
    T::Unsigned: Bounded + HasUnsignedVersion<LeBytes=T::LeBytes, Unsigned=T::Unsigned> 
        + WrappingAdd + ToPrimitive + Sub<Output=T::Unsigned>
        + std::ops::Rem<Output=T::Unsigned> + FromPrimitive + Zero
        + std::cmp::Eq + std::ops::Div<Output=T::Unsigned>
        + Ord + std::ops::Mul<Output=T::Unsigned> + WrappingSub + Copy + std::fmt::Debug,
    std::ops::RangeInclusive<T::Unsigned>: Iterator<Item=T::Unsigned>
    {
        let bin_count = (to_u(right) - to_u(left)).to_usize().unwrap() + 1;
        let hist_wrapped =  HistogramInt::<T>::new_inclusive(left, right, bin_count);
        if hist_wrapped.is_err(){
            dbg!(&hist_wrapped);
        }
        let mut hist = hist_wrapped.unwrap();
        assert!(hist.not_inside(T::max_value()));
        assert!(hist.not_inside(T::min_value()));
        for (id, i) in (left..=right).enumerate() {
            assert!(hist.is_inside(i));
            assert_eq!(hist.is_inside(i), !hist.not_inside(i));
            assert!(hist.get_bin_index(i).unwrap() == id);
            assert_eq!(hist.distance(i), 0.0);
            assert_eq!(hist.interval_distance_overlap(i, 2), 0);
            hist.count_val(i).unwrap();
        }
        let lm1 = left - T::one();
        let rp1 = right + T::one();
        assert!(hist.not_inside(lm1));
        assert!(hist.not_inside(rp1));
        assert_eq!(hist.is_inside(lm1), !hist.not_inside(lm1));
        assert_eq!(hist.is_inside(rp1), !hist.not_inside(rp1));
        assert_eq!(hist.distance(lm1), 1.0);
        assert_eq!(hist.distance(rp1), 1.0);
        assert_eq!(hist.interval_distance_overlap(rp1, 1), 1);
        assert_eq!(hist.interval_distance_overlap(lm1, 1), 1);
        assert_eq!(hist.borders_clone().unwrap().len() - 1, hist.bin_count());
        assert_eq!(
            HistogramInt::<T>::new_inclusive(left, T::max_value(), bin_count).unwrap_err(),
            HistErrors::Overflow
        );
    }


    #[test]
    fn hist_normal()
    {
        hist_test_normal(20usize, 31usize);
        hist_test_normal(-23isize, 31isize);
        hist_test_normal(-23i16, 31);
        hist_test_normal(1u8, 3u8);
        hist_test_normal(123u128, 300u128);
        hist_test_normal(-123i128, 300i128);

        hist_test_normal(i8::MIN + 1, i8::MAX - 1);
    }
}