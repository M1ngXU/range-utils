use std::ops::{Bound, RangeBounds, RangeInclusive, Sub};

/// Basic operations (increase decrease) for numbers
pub trait BasicNum {
    const MIN_VALUE: Self;
    const MAX_VALUE: Self;
    fn dec(&self) -> Self;
    fn inc(&self) -> Self;
}
macro_rules! impl_primitive_basic_num {
    ($($t:ty),*) => {
        $(
            impl BasicNum for $t {
                const MIN_VALUE: Self = Self::MIN;
                const MAX_VALUE: Self = Self::MAX;
                fn dec(&self) -> Self {
                    self - 1
                }
                fn inc(&self) -> Self {
                    self + 1
                }
            }
        )*
    };
}
// no f32/f64 since range useless on these
impl_primitive_basic_num!(usize, isize, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

/// Note that this implementation is inefficient if cloning is extremely expensive.
pub trait RangeUtil<T: Ord + Clone + BasicNum>: Sized + Clone {
    /// Start bound inclusive
    fn starts_at(&self) -> T;
    /// End bound inclusive
    fn ends_at(&self) -> T;

    /// The length of the range
    fn len(&self) -> Option<T>
    where
        T: Sub<Output = T>,
    {
        (self.ends_at() >= self.starts_at()).then(|| self.ends_at() - self.starts_at().inc())
    }
    /// Using different name to prevent name clash, this does not require `Self: RangeBound`
    fn includes(&self, x: &T) -> bool {
        &self.starts_at() <= x && x <= &self.ends_at()
    }
    /// Whether two ranges intersect, e.g. `0..=3` and `1..=4` intersect while `0..=3` and `4..` don't
    ///
    /// This also works for "different ranges", e.g. `0..=3` and `2..` returns `true`
    fn intersects(&self, other: &impl RangeUtil<T>) -> bool {
        self.ends_at() >= other.starts_at() && self.starts_at() <= other.ends_at()
    }
    /// The intersection of two ranges, e.g. `0..=3` and `1..=4` is `1..=3`
    ///
    /// This also works for "different ranges", e.g. `0..=3` and `2..` is `1..=3`
    fn intersection(&self, other: &impl RangeUtil<T>) -> Option<RangeInclusive<T>> {
        self.intersects(other)
            .then(|| self.starts_at().max(other.starts_at())..=self.ends_at().min(other.ends_at()))
    }
    /// The result of substracting `other` from `self`, e.g. `0..=3`\`1..=4` is `(0..=0, None)`
    ///
    /// If there are two sets representing the result, then the smaller range comes first. If only one range represents the result, then either result may be `None` (implementation detail, may change in the future).
    ///
    /// This also works for "different ranges", e.g. `0..=3`\`2..` is `0..=1`
    fn setminus(
        &self,
        other: &impl RangeUtil<T>,
    ) -> (Option<RangeInclusive<T>>, Option<RangeInclusive<T>>) {
        let Some(other) = self.intersection(other) else {
            return (
                Some(self.starts_at().clone()..=self.ends_at().clone()),
                None,
            );
        };
        let (a, b) = (self.starts_at().clone(), self.ends_at().clone());
        let (c, d) = (other.start().clone(), other.ends_at().clone());
        (
            (self.includes(&c))
                .then(|| a..=c.dec())
                .filter(|r| !r.is_empty()),
            self.includes(&d)
                .then(|| d.inc()..=b)
                .filter(|r| !r.is_empty()),
        )
    }
}
impl<T: Ord + Clone + BasicNum, R: RangeBounds<T> + Clone> RangeUtil<T> for R {
    fn starts_at(&self) -> T {
        match self.start_bound() {
            Bound::Excluded(x) => x.inc(),
            Bound::Included(x) => x.clone(),
            Bound::Unbounded => T::MIN_VALUE,
        }
    }
    fn ends_at(&self) -> T {
        match self.end_bound() {
            Bound::Excluded(x) => x.dec(),
            Bound::Included(x) => x.clone(),
            Bound::Unbounded => T::MAX_VALUE,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::RangeFull;

    use crate::RangeUtil;

    #[test]
    fn test_intersection_range_inclusive() {
        assert_eq!((0..=3).intersection(&(1..=2)), Some(1..=2));
        assert_eq!((0..=3).intersection(&(1..=30)), Some(1..=3));
        assert_eq!((0..=3).intersection(&(-10..1)), Some(0..=0));
        assert_eq!((0..=3).intersection(&(-10..=1)), Some(0..=1));
        assert_eq!((0..=3).intersection(&(-10..=-1)), None);
        assert_eq!((0..=3).intersection(&(4..=10)), None);
    }

    #[test]
    fn test_difference_range_inclusive() {
        assert_eq!((0..=3).setminus(&(4..=100)), (Some(0..=3), None));
        assert_eq!((0..=3).setminus(&(-100..=-1)), (Some(0..=3), None));
        assert_eq!((0..=3).setminus(&(1..=2)), (Some(0..=0), Some(3..=3)));
        assert_eq!((0..=3).setminus(&(0..=2)), (None, Some(3..=3)));
        assert_eq!((0..=3).setminus(&(1..=3)), (Some(0..=0), None));
    }

    #[test]
    fn test_from_incl() {
        assert_eq!((0..).starts_at(), 0);
        assert_eq!((0..1).starts_at(), 0);
        assert_eq!((0..=1).starts_at(), 0);
        assert_eq!((..=10).starts_at(), 0usize);
        assert_eq!((..=10).starts_at(), isize::MIN);
        assert_eq!(RangeUtil::<usize>::starts_at(&RangeFull), 0);
        assert_eq!(RangeUtil::<isize>::starts_at(&RangeFull), isize::MIN);
    }

    #[test]
    fn test_to_incl() {
        assert_eq!((0..).ends_at(), usize::MAX);
        assert_eq!((0..1).ends_at(), 0);
        assert_eq!((0..=1).ends_at(), 1);
        assert_eq!((..=10).ends_at(), 10);
        assert_eq!((10..).ends_at(), isize::MAX);
        assert_eq!(RangeUtil::<usize>::ends_at(&RangeFull), usize::MAX);
        assert_eq!(RangeUtil::<isize>::ends_at(&RangeFull), isize::MAX);
    }
}
