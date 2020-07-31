use num_traits::{int::*, float::*, ops::{checked::*, saturating::*}, cast::*, identities::*};
use crate::sampling::histogram::*;
use std::{borrow::*, ops::*};
#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};


/// Generic Histogram struct
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct HistogramFloat<T>
{
    bin_borders: Vec<T>,
    hist: Vec<usize>,
}

impl<T> HistogramFloat<T>{
    /// similar to `self.borders_clone` but does not allocate memory
    pub fn borders(&self) -> &Vec<T>
    {
        &self.bin_borders
    }
}

impl<T> HistogramFloat<T> 
where T: Float + PartialOrd + FromPrimitive {
    /// # Create a new Historgram
    /// * right exclusive, left inclusive
    /// * if you want `right` to behave (almost) the same as an inclusive border,
    /// consider using `new(left, right + T::EPSILON, bins)` (make sure, that adding Epsilon actually changes the value!)
    pub fn new(left: T, right: T, bins: usize) -> Result<Self, HistErrors>
    {
        if left >= right {
            return Err(HistErrors::IntervalWidthZero);
        }
        else if bins < 1 {
            return Err(HistErrors::NoBins);
        }
        if !left.is_finite() || !right.is_finite() {
            return Err(HistErrors::InvalidVal);
        }

        let bins_as_t = match T::from_usize(bins) {
            Some(val) => val,
            None => return Err(HistErrors::UsizeCastError),
        };

        let bin_size = (right - left) / bins_as_t;
        let hist = vec![0; bins];
        let mut bin_borders = Vec::with_capacity(bins + 1);
        bin_borders.extend((0..bins)
            .map(|val| bin_size.mul_add(T::from_usize(val).unwrap(), left)) 
        );
        bin_borders.push(right);
        Ok(
            Self{
                bin_borders,
                hist
            }
        )
    }

    /// Returns the length of the interval
    pub fn interval_length(&self) -> T
    {
        self.get_right() - self.get_left()
    }
}

impl<T> Histogram for HistogramFloat<T>
{
    #[inline]
    fn bin_count(&self) -> usize {
        self.hist.len()
    }

    #[inline]
    fn hist(&self) -> &Vec<usize> {
        &self.hist
    }

    fn count_index(&mut self, index: usize) -> Result<(), HistErrors> {
        if index < self.bin_count()
        {
            self.hist[index] += 1;
            Ok(())
        } else {
            Err(HistErrors::OutsideHist)
        }
    }

    fn reset(&mut self) {
        for i in 0..self.bin_count() {
            self.hist[i] = 0;
        }
    }
}

impl<T> HistogramVal<T> for HistogramFloat<T>
where T: Float + Zero + NumCast {

    fn distance(&self, val: T) -> f64 {
        if self.is_inside(val) {
            0.0
        } else if !val.is_finite() {
            f64::INFINITY
        } else if val < self.get_left() {
            (self.get_left() - val).to_f64().unwrap()
        } else {
            (val - self.get_right() + T::epsilon())
                .to_f64()
                .unwrap()
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

    fn is_inside<V: Borrow<T>>(&self, val: V) -> bool {
        *val.borrow() >= self.bin_borders[0] 
            && *val.borrow() < self.bin_borders[self.bin_borders.len() - 1]
    }

    fn not_inside<V: Borrow<T>>(&self, val: V) -> bool {
        !(*val.borrow()).is_finite() 
            || *val.borrow() < self.bin_borders[0] 
            || *val.borrow() >= self.bin_borders[self.bin_borders.len() - 1]
    }


    fn get_bin_index<V: Borrow<T>>(&self, val: V) -> Result<usize, HistErrors>
    {
        let val = val.borrow();
        if !val.is_finite(){
            return Err(HistErrors::InvalidVal);
        }
        else if self.is_inside(val)
        {
            let search_res = self.bin_borders.binary_search_by(
                |v|
                v.partial_cmp(val).expect("Should never be NaN")
            );
            match search_res
            {
                Result::Ok(index) => {
                    Ok(index)
                },
                Result::Err(index_p1) => {
                    Ok(index_p1 - 1)
                }
            }
        }
        else {
            Err(HistErrors::OutsideHist)
        } 
    }

    /// consider using `self.borders()`
    fn borders_clone(&self) -> Result<Vec<T>, HistErrors> {
        Ok(self.bin_borders.clone())
    }
}

impl<T> HistogramIntervalDistance<T> for HistogramFloat<T> 
where T: Float + FromPrimitive + Zero + NumCast
{
    fn interval_distance_overlap(&self, val: T, overlap: usize) -> usize {
        debug_assert!(overlap > 0);
        debug_assert!(self.interval_length() > T::zero());
        debug_assert!(val.is_finite());
        if self.not_inside(val) {
            let num_bins_overlap = self.bin_count() / overlap;
            let dist = 
            if val < self.get_left() { 
                let tmp = self.get_left() - val;
                (tmp / self.interval_length()).floor()
            } else {
                let tmp = val - self.get_right();
                (tmp / self.interval_length()).ceil()
            };
            let int_dist = dist.to_usize().unwrap();
            1 + int_dist / num_bins_overlap
        } else {
            0
        }
    }
}

/// Histogram for binning `f32` - alias for `HistogramFloat<f32>`
pub type HistF32 = HistogramFloat<f32>;

/// Histogram for binning `f64` - alias for `HistogramFloat<f64>`
pub type HistF64 = HistogramFloat<f64>;

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
where T: PartialOrd + ToPrimitive + FromPrimitive + CheckedAdd + One
        + Sub<T, Output=T> + Mul<T, Output=T> + Zero + Copy + std::fmt::Debug,
    std::ops::RangeInclusive<T>: Iterator<Item=T>
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
        let border_difference = match (right - left).to_usize(){
            Some(val) => val,
            None => return Err(HistErrors::UsizeCastError),
        };
        if border_difference % bins != 0 {
            return Err(HistErrors::ModuloError);
        }

        let bin_size = match T::from_usize(border_difference / bins){
            Some(val) => val,
            None => return Err(HistErrors::CastError),
        };
        if bin_size <= T::zero() {
            return Err(HistErrors::IntervalWidthZero);
        }
        
        let hist = vec![0; bins];
        let bins = match T::from_usize(bins) {
            Some(val) => val,
            None => return Err(HistErrors::CastError),
        };

        let bin_borders: Vec<_> = (T::zero()..=bins)
            .map(|val| left + val * bin_size)
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

    fn count_index(&mut self, index: usize) -> Result<(), HistErrors> {
        if index < self.bin_count()
        {
            self.hist[index] += 1;
            Ok(())
        } else {
            Err(HistErrors::OutsideHist)
        }
    }

    fn reset(&mut self) {
        // compiles down to memset :)
        self.hist
            .iter_mut()
            .for_each(|val| *val = 0);
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
    fn interval_distance_overlap(&self, val: T, overlap: usize) -> usize {
        debug_assert!(overlap > 0);
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


/// # Faster version of HistogramGeneric for Integers
/// provided the bins should be: (left, left +1, ..., right - 1)
/// then you should use this version!
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct HistogramFast<T> {
    left: T, 
    right: T,
    hist: Vec<usize>,
}

impl<T> HistogramFast<T> 
    where T: PrimInt + CheckedSub + ToPrimitive + CheckedAdd + One 
{
    /// # Create a new interval
    /// * same as `Self::new_inclusive(left, right - 1)` though with checks
    pub fn new(left: T, right: T) -> Result<Self, HistErrors>
    {
        let right = match right.checked_sub(&T::one()){
            Some(res) => res,
            None => return Err(HistErrors::Underflow),
        };
        Self::new_inclusive(left, right)
    }

    /// # Create new histogram with inclusive borders
    /// * Err if `left > right`
    /// * left is inclusive, right is exclusive
    pub fn new_inclusive(left: T, right: T) -> Result<Self, HistErrors>
    {
        if left > right {
            Err(HistErrors::OutsideHist)
        } else {
            let size = match right.checked_sub(&left){
                None => return Err(HistErrors::Underflow),
                Some(res) => res
            };
            let size = match size.to_usize() {
                None => return Err(HistErrors::UsizeCastError),
                Some(res) => res + 1,
            };

            Ok(
                Self{
                    left,
                    right,
                    hist: vec![0; size],
                }
            )
        }
    }
}
/// Histogram for binning `usize`- alias for `HistogramFast<usize>`
pub type HistUsizeFast = HistogramFast<usize>;
/// Histogram for binning `u64 - alias for `HistogramFast<u64>`
pub type HistU64Fast = HistogramFast<u64>;
/// Histogram for binning `u32` - alias for `HistogramFast<u32>`
pub type HistU32Fast = HistogramFast<u32>;
/// Histogram for binning `u16` - alias for `HistogramFast<u16>`
pub type HistU16Fast = HistogramFast<u16>;
/// Histogram for binning `u8` - alias for `HistogramFast<u8>`
pub type HistU8Fast = HistogramFast<u8>;

/// Histogram for binning `isize` - alias for `HistogramFast<isize>`
pub type HistIsizeFast = HistogramFast<isize>;
/// Histogram for binning `i64` - alias for `HistogramFast<i64>`
pub type HistI64Fast = HistogramFast<i64>;
/// Histogram for binning `i32` - alias for `HistogramFast<i32>`
pub type HistI32Fast = HistogramFast<i32>;
/// Histogram for binning `i16` - alias for `HistogramFast<i16>`
pub type HistI16Fast = HistogramFast<i16>;
/// Histogram for binning `i8` - alias for `HistogramFastiu8>`
pub type HistI8Fast = HistogramFast<i8>;


impl<T> Histogram for HistogramFast<T> 
{

    fn count_index(&mut self, index: usize) -> Result<(), HistErrors> {
        match self.hist.get_mut(index) {
            None => Err(HistErrors::OutsideHist),
            Some(val) => {
                *val += 1;
                Ok(())
            },
        }
    }

    #[inline]
    fn hist(&self) -> &Vec<usize> {
        &self.hist
    }

    #[inline]
    fn bin_count(&self) -> usize {
        self.hist.len()
    }

    fn reset(&mut self) {
        for i in 0..self.bin_count(){
            self.hist[i] = 0;
        }
    }
}

impl<T> HistogramVal<T> for HistogramFast<T>
where T: PartialOrd + CheckedSub + CheckedAdd + One + Saturating + NumCast + Copy,
    std::ops::RangeInclusive<T>: Iterator<Item=T>
{
    #[inline]
    fn get_left(&self) -> T {
        self.left
    }

    #[inline]
    fn get_right(&self) -> T {
        self.right
    }

    fn distance(&self, val: T) -> f64 {
        if self.not_inside(val) {
            let dist = if val < self.get_left() {
                self.get_left() - val
            } else {
                val.saturating_sub(self.right)
            };
            dist.to_f64()
                .unwrap_or(f64::INFINITY)
        } else {
            0.0
        }
    }

    fn get_bin_index<V: Borrow<T>>(&self, val: V) -> Result<usize, HistErrors> {
        let val = *val.borrow();
        if val <= self.right{
            match val.checked_sub(&self.left) {
                None => Err(HistErrors::OutsideHist),
                Some(index) => Ok(index.to_usize().unwrap())
            }
        } else {
            Err(HistErrors::OutsideHist)
        }
    }

    /// * returns `Err(Overflow)` if right border is `T::MAX`
    /// * returns borders otherwise
    fn borders_clone(&self) -> Result<Vec<T>, HistErrors> {
        let right = self.right.checked_add(&T::one())
            .ok_or(HistErrors::Overflow)?;
        Ok((self.left..=right).collect())
    }

    #[inline]
    fn is_inside<V: Borrow<T>>(&self, val: V) -> bool {
        let val = *val.borrow();
        val >= self.left && val <= self.right
    }

    #[inline]
    fn not_inside<V: Borrow<T>>(&self, val: V) -> bool {
        let val = *val.borrow();
        val > self.right || val < self.left
    }

    #[inline]
    fn count_val<V: Borrow<T>>(&mut self, val: V) -> Result<usize, HistErrors> {
        let index = self.get_bin_index(val)?;
        self.hist[index] += 1;
        Ok(index)
    }
}

impl<T> HistogramIntervalDistance<T> for HistogramFast<T> 
where Self: HistogramVal<T>,
    T: PartialOrd + std::ops::Sub<Output=T> + NumCast + Copy
{
    fn interval_distance_overlap(&self, val: T, overlap: usize) -> usize {
        debug_assert!(overlap > 0);
        if self.not_inside(val) {
            let num_bins_overlap = 1usize.max(self.bin_count() / overlap);
            let dist = 
            if val < self.left { 
                self.left - val
            } else {
                val - self.right
            };
            1 + dist.to_usize().unwrap() / num_bins_overlap
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests{
    use rand_pcg::Pcg64Mcg;
    use crate::rand::{distributions::*, SeedableRng};
    use super::*;
    use num_traits::Bounded;
    #[test]
    fn f64_hist()
    {
        let rng = Pcg64Mcg::new(0xcafef00dd15ea5e5);
        let dist = Uniform::new(f64::EPSILON, 1.0);
        let mut iter = dist.sample_iter(rng);

        for i in 1..100 {
            let left = iter.next().unwrap();
            let right = left + iter.next().unwrap();

            let hist = HistogramFloat::<f64>::new(left, right, i).unwrap();

            assert_eq!(left, hist.get_left(), "i={}", i);
            assert_eq!(right, hist.get_right(), "i={}", i);
            assert_eq!(i+1, hist.borders().len(), "i={}", i);

        }
    }


    fn hist_test_fast<T>(left: T, right: T)
    where T: PrimInt + num_traits::Bounded + PartialOrd + CheckedSub + One + Saturating + NumCast + Copy,
    std::ops::RangeInclusive<T>: Iterator<Item=T>
    {
        let mut hist = HistogramFast::<T>::new_inclusive(left, right).unwrap();
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
    }

    fn hist_test_normal<T>(left: T, right: T)
    where T: PrimInt + num_traits::Bounded + PartialOrd + CheckedSub 
        + One + Saturating + NumCast + Copy + FromPrimitive + Bounded + std::fmt::Debug,
    std::ops::RangeInclusive<T>: Iterator<Item=T>
    {
        let bin_count = (right - left).to_usize().unwrap() + 1;
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
    fn hist_fast()
    {
        hist_test_fast(20usize, 31usize);
        hist_test_fast(-23isize, 31isize);
        hist_test_fast(-23i16, 31);
        hist_test_fast(1u8, 3u8);
    }

    #[test]
    fn hist_normal()
    {
        hist_test_normal(20usize, 31usize);
        hist_test_normal(-23isize, 31isize);
        hist_test_normal(-23i16, 31);
        hist_test_normal(1u8, 3u8);
    }

    fn hist_test_float<T>(left: T, right: T, bin_count: usize)
    where T: Float + num_traits::Bounded + PartialOrd 
        + One + NumCast + Copy + FromPrimitive + Bounded + std::fmt::Debug
        + PartialOrd,
    {

        let hist_wrapped =  HistogramFloat::<T>::new(left, right, bin_count);
        if hist_wrapped.is_err(){
            dbg!(&hist_wrapped);
        }
        let hist = hist_wrapped.unwrap();
        assert!(hist.not_inside(T::infinity()));
        assert!(hist.not_inside(T::nan()));
        let len = hist.borders().len();
        
        for (id, border) in hist.borders()
            .iter()
            .take(len - 1)
            .enumerate()
        {
            assert!(hist.is_inside(border));
            assert_eq!(hist.is_inside(border), !hist.not_inside(border));
            assert_eq!(hist.get_bin_index(border).unwrap(), id);
        }
        
        let last_border = hist.borders()[len - 1];
        assert!(hist.not_inside(last_border));
        assert_eq!(hist.is_inside(last_border), !hist.not_inside(last_border));
        assert!(hist.get_bin_index(last_border).is_err());
        

        for (id, border) in hist.borders()
            .iter()
            .skip(1)
            .enumerate()
        {
            let mut m_epsilon = *border;
            for mut i in 1..{
                if i > 100 {
                    i = i * i;
                }
                m_epsilon = T::epsilon().mul_add(
                    T::from_isize(-i).unwrap(), 
                    *border
                );
                if m_epsilon < *border {
                    break;
                }
            }
            assert!(hist.is_inside(m_epsilon));
            assert_eq!(hist.get_bin_index(m_epsilon).unwrap(), id);
        }
       
        assert_eq!(
            HistErrors::InvalidVal,
            HistogramFloat::<T>::new(T::nan(), right, bin_count).unwrap_err()
        );
        assert_eq!(
            HistErrors::InvalidVal,
            HistogramFloat::<T>::new(left, T::nan(), bin_count).unwrap_err()
        );
        assert_eq!(
            HistErrors::InvalidVal,
            HistogramFloat::<T>::new(left, T::infinity(), bin_count).unwrap_err()
        );
        assert_eq!(
            HistErrors::InvalidVal,
            HistogramFloat::<T>::new(T::neg_infinity(), right, bin_count).unwrap_err()
        );
    }

    #[test]
    fn hist_float()
    { 
        let mut rng = Pcg64Mcg::new(0xcafef00dd15ea5e5);
        let dist = Uniform::new(1usize, 111);
        let mut iter = dist.sample_iter(
            Pcg64Mcg::from_rng(&mut rng).unwrap()
        );
        hist_test_float(20.0, 31.0, iter.next().unwrap());
        hist_test_float(-23.0f32, 31.1232f32, iter.next().unwrap());
        hist_test_float(-13.0f32, 31.4657f32, iter.next().unwrap());
        hist_test_float(1.0f64, 3f64, iter.next().unwrap());

        let dist2 = Uniform::new(0.0, 76257f64);
        for _ in 0..10 {
            let (left, right) = loop{
                let left = dist2.sample(&mut rng);
                let right = left + dist2.sample(&mut rng);
                if left.is_finite() && right.is_finite(){
                    break (left, right);
                }
            };
            hist_test_float(left, right, iter.next().unwrap());
        }
    }

}