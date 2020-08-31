use num_traits::{ops::wrapping::*, Bounded};

pub trait HasUnsignedVersion {
    type Unsigned;
    type LeBytes;
    fn to_le_bytes(self) -> Self::LeBytes;
    fn from_le_bytes(bytes: Self::LeBytes) -> Self;
}

impl HasUnsignedVersion for u8 {
    type Unsigned = u8;
    type LeBytes = [u8; 1];

    #[inline(always)]
    fn to_le_bytes(self) -> Self::LeBytes {
        self.to_le_bytes()
    }

    #[inline(always)]
    fn from_le_bytes(bytes: Self::LeBytes) -> Self {
        Self::from_le_bytes(bytes)
    }
}

impl HasUnsignedVersion for u16 {
    type Unsigned = u16;
    type LeBytes = [u8; 2];

    #[inline(always)]
    fn to_le_bytes(self) -> Self::LeBytes {
        self.to_le_bytes()
    }

    #[inline(always)]
    fn from_le_bytes(bytes: Self::LeBytes) -> Self {
        Self::from_le_bytes(bytes)
    }
}

impl HasUnsignedVersion for u32 {
    type Unsigned = u32;
    type LeBytes = [u8; 4];

    #[inline(always)]
    fn to_le_bytes(self) -> Self::LeBytes {
        self.to_le_bytes()
    }

    #[inline(always)]
    fn from_le_bytes(bytes: Self::LeBytes) -> Self {
        Self::from_le_bytes(bytes)
    }
}

impl HasUnsignedVersion for u64 {
    type Unsigned = u64;
    type LeBytes = [u8; 8];

    #[inline(always)]
    fn to_le_bytes(self) -> Self::LeBytes {
        self.to_le_bytes()
    }

    #[inline(always)]
    fn from_le_bytes(bytes: Self::LeBytes) -> Self {
        Self::from_le_bytes(bytes)
    }
}

impl HasUnsignedVersion for u128 {
    type Unsigned = u128;
    type LeBytes = [u8; 16];

    #[inline(always)]
    fn to_le_bytes(self) -> Self::LeBytes {
        self.to_le_bytes()
    }

    #[inline(always)]
    fn from_le_bytes(bytes: Self::LeBytes) -> Self {
        Self::from_le_bytes(bytes)
    }
}

impl HasUnsignedVersion for usize {
    type Unsigned = usize;
    type LeBytes = [u8; 8];

    #[inline(always)]
    fn to_le_bytes(self) -> Self::LeBytes {
        self.to_le_bytes()
    }

    #[inline(always)]
    fn from_le_bytes(bytes: Self::LeBytes) -> Self {
        Self::from_le_bytes(bytes)
    }
}

impl HasUnsignedVersion for i8 {
    type Unsigned = u8;
    type LeBytes = [u8; 1];

    #[inline(always)]
    fn to_le_bytes(self) -> Self::LeBytes {
        self.to_le_bytes()
    }

    #[inline(always)]
    fn from_le_bytes(bytes: Self::LeBytes) -> Self {
        Self::from_le_bytes(bytes)
    }
}

impl HasUnsignedVersion for i16 {
    type Unsigned = u16;
    type LeBytes = [u8; 2];

    #[inline(always)]
    fn to_le_bytes(self) -> Self::LeBytes {
        self.to_le_bytes()
    }

    #[inline(always)]
    fn from_le_bytes(bytes: Self::LeBytes) -> Self {
        Self::from_le_bytes(bytes)
    }
}

impl HasUnsignedVersion for i32 {
    type Unsigned = u32;
    type LeBytes = [u8; 4];

    #[inline(always)]
    fn to_le_bytes(self) -> Self::LeBytes {
        self.to_le_bytes()
    }

    #[inline(always)]
    fn from_le_bytes(bytes: Self::LeBytes) -> Self {
        Self::from_le_bytes(bytes)
    }
}

impl HasUnsignedVersion for i64 {
    type Unsigned = u64;
    type LeBytes = [u8; 8];

    #[inline(always)]
    fn to_le_bytes(self) -> Self::LeBytes {
        self.to_le_bytes()
    }

    #[inline(always)]
    fn from_le_bytes(bytes: Self::LeBytes) -> Self {
        Self::from_le_bytes(bytes)
    }
}

impl HasUnsignedVersion for i128 {
    type Unsigned = u128;
    type LeBytes = [u8; 16];

    #[inline(always)]
    fn to_le_bytes(self) -> Self::LeBytes {
        self.to_le_bytes()
    }

    #[inline(always)]
    fn from_le_bytes(bytes: Self::LeBytes) -> Self {
        Self::from_le_bytes(bytes)
    }
}

impl HasUnsignedVersion for isize {
    type Unsigned = usize;
    type LeBytes = [u8; 8];

    #[inline(always)]
    fn to_le_bytes(self) -> Self::LeBytes {
        self.to_le_bytes()
    }
    
    #[inline(always)]
    fn from_le_bytes(bytes: Self::LeBytes) -> Self {
        Self::from_le_bytes(bytes)
    }
}

#[inline(always)]
pub(crate) fn to_u<T>(v: T) -> T::Unsigned
where T: num_traits::Bounded + HasUnsignedVersion,
    T::Unsigned: num_traits::Bounded + HasUnsignedVersion<LeBytes=T::LeBytes> + WrappingAdd
{
    let u = T::Unsigned::from_le_bytes(v.to_le_bytes());
    u.wrapping_add(&T::Unsigned::from_le_bytes(T::min_value().to_le_bytes()))
}

#[inline(always)]
pub(crate) fn from_u<T, V>(u: T) -> V
where T: num_traits::Bounded + HasUnsignedVersion + WrappingSub + Bounded,
    T::Unsigned: num_traits::Bounded + HasUnsignedVersion<LeBytes=T::LeBytes> + WrappingAdd,
    V: HasUnsignedVersion<LeBytes=T::LeBytes> + Bounded
{
    let u = u.wrapping_sub(&T::from_le_bytes(V::min_value().to_le_bytes()));
    V::from_le_bytes(u.to_le_bytes())
}


#[cfg(test)]
mod tests{
    use rand_pcg::Pcg64Mcg;
    use crate::rand::{SeedableRng, distributions::*};
    use super::*;


    #[test]
    fn convert_and_back_ord()
    {
        let rng = Pcg64Mcg::seed_from_u64(2747);
        let dist = Uniform::new_inclusive(i8::MIN, i8::MAX);
        let mut iter = dist.sample_iter(rng);

        for _ in 0..1000
        {
            let a = iter.next().unwrap();
            let b = iter.next().unwrap();
            assert_eq!(a < b, to_u(a) < to_u(b));
        }
    }
    #[test]
    fn convert_and_back_i8()
    {
        let rng = Pcg64Mcg::seed_from_u64(2747);
        let dist = Uniform::new_inclusive(i8::MIN, i8::MAX);
        let iter = dist.sample_iter(rng);

        for i in iter.take(10000)
        {
            assert_eq!(i, from_u::<_, i8>(to_u(i)));
        }
    }
    #[test]
    fn convert_and_back_i16()
    {
        let rng = Pcg64Mcg::seed_from_u64(2736746347);
        let dist = Uniform::new_inclusive(i16::MIN, i16::MAX);
        let iter = dist.sample_iter(rng);

        for i in iter.take(10000)
        {
            assert_eq!(i, from_u::<_, i16>(to_u(i)));
        }
    }

    #[test]
    fn convert_and_back_isize()
    {
        let rng = Pcg64Mcg::seed_from_u64(27367463247);
        let dist = Uniform::new_inclusive(isize::MIN, isize::MAX);
        let iter = dist.sample_iter(rng);

        for i in iter.take(10000)
        {
            assert_eq!(i, from_u::<_, isize>(to_u(i)));
        }
    }

    #[test]
    fn convert_and_back_u128()
    {
        let rng = Pcg64Mcg::seed_from_u64(273674693247);
        let dist = Uniform::new_inclusive(u128::MIN, u128::MAX);
        let iter = dist.sample_iter(rng);

        for i in iter.take(10000)
        {
            assert_eq!(i, from_u::<_, u128>(to_u(i)));
        }
    }



    #[test]
    fn convert_and_back_i128()
    {
        let rng = Pcg64Mcg::seed_from_u64(2723674693247);
        let dist = Uniform::new_inclusive(i128::MIN, i128::MAX);
        let iter = dist.sample_iter(rng);

        for i in iter.take(10000)
        {
            assert_eq!(i, from_u::<_, i128>(to_u(i)));
        }
    }
}