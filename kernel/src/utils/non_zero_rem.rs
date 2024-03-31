use core::num::NonZero;

pub trait NonZeroRem {
    fn non_zero_rem(self, rhs: Self) -> Self;
}

macro_rules! impl_non_zero_rem {
    ($t:ty) => {
        impl NonZeroRem for NonZero<$t> {
            fn non_zero_rem(self, rhs: Self) -> Self {
                let left: $t = self.into();
                let right: $t = rhs.into();
                let result = left % right;
                let non_zero_result = match result {
                    0 => right,
                    n => n,
                };
                unsafe { NonZero::<$t>::new_unchecked(non_zero_result) }
            }
        }
    };
}

impl_non_zero_rem!(u8);
impl_non_zero_rem!(u16);
impl_non_zero_rem!(u32);
impl_non_zero_rem!(u64);
impl_non_zero_rem!(usize);

impl_non_zero_rem!(i8);
impl_non_zero_rem!(i16);
impl_non_zero_rem!(i32);
impl_non_zero_rem!(i64);
impl_non_zero_rem!(isize);
