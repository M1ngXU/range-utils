use std::ops::{Bound, RangeBounds, RangeInclusive};

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
    fn from_incl(&self) -> T;
    /// End bound inclusive
    fn to_incl(&self) -> T;

    /// Using different name to prevent name clash, this does not require `Self: RangeBound`
    fn includes(&self, x: &T) -> bool {
        &self.from_incl() <= x && x <= &self.to_incl()
    }
    /// Whether two ranges intersect, e.g. `0..=3` and `1..=4` intersect while `0..=3` and `4..` don't
    ///
    /// This also works for "different ranges", e.g. `0..=3` and `2..` returns `true`
    fn intersects<O: RangeUtil<T>>(&self, other: &O) -> bool {
        self.to_incl() >= other.from_incl() && self.from_incl() <= other.to_incl()
    }
    /// The intersection of two ranges, e.g. `0..=3` and `1..=4` is `1..=3`
    ///
    /// This also works for "different ranges", e.g. `0..=3` and `2..` is `1..=3`
    fn intersection<O: RangeUtil<T>>(&self, other: &O) -> Option<RangeInclusive<T>> {
        self.intersects(other)
            .then(|| self.from_incl().max(other.from_incl())..=self.to_incl().min(other.to_incl()))
    }
    /// The result of substracting `other` from `self`, e.g. `0..=3`\`1..=4` is `(0..=0, None)`
    ///
    /// If there are two sets representing the result, then the smaller range comes first. If only one range represents the result, then either result may be `None` (implementation detail, may change in the future).
    ///
    /// This also works for "different ranges", e.g. `0..=3`\`2..` is `0..=1`
    fn setminus<O: RangeUtil<T>>(
        &self,
        other: &O,
    ) -> (Option<RangeInclusive<T>>, Option<RangeInclusive<T>>) {
        let Some(other) = self.intersection(other) else {
            return (
                Some(self.from_incl().clone()..=self.to_incl().clone()),
                None,
            );
        };
        let (a, b) = (self.from_incl().clone(), self.to_incl().clone());
        let (c, d) = (other.start().clone(), other.to_incl().clone());
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
    fn from_incl(&self) -> T {
        match self.start_bound() {
            Bound::Excluded(x) => x.inc(),
            Bound::Included(x) => x.clone(),
            Bound::Unbounded => T::MIN_VALUE,
        }
    }
    fn to_incl(&self) -> T {
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
        assert_eq!((0..).from_incl(), 0);
        assert_eq!((0..1).from_incl(), 0);
        assert_eq!((0..=1).from_incl(), 0);
        assert_eq!((..=10).from_incl(), 0usize);
        assert_eq!((..=10).from_incl(), isize::MIN);
        assert_eq!(RangeUtil::<usize>::from_incl(&RangeFull), 0);
        assert_eq!(RangeUtil::<isize>::from_incl(&RangeFull), isize::MIN);
    }

    #[test]
    fn test_to_incl() {
        assert_eq!((0..).to_incl(), usize::MAX);
        assert_eq!((0..1).to_incl(), 0);
        assert_eq!((0..=1).to_incl(), 1);
        assert_eq!((..=10).to_incl(), 10);
        assert_eq!((10..).to_incl(), isize::MAX);
        assert_eq!(RangeUtil::<usize>::to_incl(&RangeFull), usize::MAX);
        assert_eq!(RangeUtil::<isize>::to_incl(&RangeFull), isize::MAX);
    }
}
