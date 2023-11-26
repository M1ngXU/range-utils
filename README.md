# range-utils

## Motivation

The standard library has `6` (or even more?) different types of ranges, e.g. `0..=256`, `..=256`, `0..`, `..` represent all the same range in `u8` (ranges like `0..3` and `..3` are some more ranges not listed before). The only "utililty" provided by the standard library is [RangeBounds](https://doc.rust-lang.org/std/ops/trait.RangeBounds.html), which is quite useless if a more advances/generic use is required. For instance, `.start_bound()` does not return the actual `start_bound` if the start is unbounded, making it harder to implement functions for different range types.

## Usage

This crate includes the trait `RangeUtil<T>` with the following methods:
- `starts_at(&self) -> T`: inclusive start bound, e.g. `(0..3).starts_at() == 0`, `(..3_u8).starts_at() == 0`
- `ends_at(&self) -> T`: inclusive end bound, e.g. `(0..3).ends_at() == 2`, `(0..=2).ends_at() == 2`, `(..=2).ends_at() == 2`, ...

The following methods have a default implementation (that does probably not need to be changed):
- `includes(&self, &T) -> bool`: `.contains(&T)` from `RangeBounds<T>` from the standard library, but implemented with `starts_at()` and `ends_at()`
- `intersects(&self, other: &impl RangeUtil<T>) -> bool`: whether two ranges intersect, e.g. `0..=2` and `2..3` do, while `0..2` and `2..3` don't
- `intersection(&self, other: &impl RangeUtil<T>) -> Option<RangeInclusive<T>>`: the intersection of two ranges with an inclusive range returned, e.g. `(0..=3).intersects(&(1..)) == Some(1..=3)`
- `setminus(&self, other: &impl RangeUtil<T>) -> (Option<RangeInclusive<T>>, Option<RangeInclusive<T>>)`: the set `self` without the elements in `other`, e.g. `(..100u8).setminus(&(50..)) == (Some(0..=50), None)`, or `(..100u8).setminus(&(25..75)) == (Some(0..=24), Some(75..=99))`
