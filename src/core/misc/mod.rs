//! Miscellaneous implementations and structures that could be moved to crates in the future,
//! in true LeftPad fashion.

pub mod range_ext {
    //! A [helper](RangeBoundsExt) trait and implementations for ranges with owned bounds.
    #[cfg(test)]
    use std::ops::RangeBounds;
    use std::ops::{Bound, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
    /// A helper to obtain an owned start and end to a [`RangeBounds`](std::ops::RangeBounds).
    pub trait RangeBoundsExt<T> {
        fn into_bounds(self) -> OwnedRangeBounds<T>;
    }
    #[derive(Debug, Eq, PartialEq)]
    pub struct OwnedRangeBounds<T> {
        pub start: Bound<T>,
        pub end: Bound<T>,
    }
    #[cfg(test)]
    fn cloned<T>(bound: Bound<&T>) -> Bound<T>
    where
        T: Clone,
    {
        match bound {
            Bound::Included(t) => Bound::Included(t.clone()),
            Bound::Excluded(t) => Bound::Excluded(t.clone()),
            Bound::Unbounded => Bound::Unbounded,
        }
    }
    impl<T> RangeBoundsExt<T> for OwnedRangeBounds<T> {
        fn into_bounds(self) -> OwnedRangeBounds<T> {
            self
        }
    }
    impl<T> RangeBoundsExt<T> for Range<T> {
        fn into_bounds(self) -> OwnedRangeBounds<T> {
            OwnedRangeBounds {
                start: Bound::Included(self.start),
                end: Bound::Excluded(self.end),
            }
        }
    }
    #[test]
    fn range() {
        let r = 3..5;
        assert_eq!(
            OwnedRangeBounds::<u32> {
                start: cloned(r.start_bound()),
                end: cloned(r.end_bound()),
            },
            r.into_bounds(),
        );
    }
    impl<T> RangeBoundsExt<T> for RangeFrom<T> {
        fn into_bounds(self) -> OwnedRangeBounds<T> {
            OwnedRangeBounds {
                start: Bound::Included(self.start),
                end: Bound::Unbounded,
            }
        }
    }
    #[test]
    fn range_from() {
        let r = 2..;
        assert_eq!(
            OwnedRangeBounds::<u32> {
                start: cloned(r.start_bound()),
                end: cloned(r.end_bound()),
            },
            r.into_bounds(),
        );
    }
    impl<T> RangeBoundsExt<T> for RangeFull {
        fn into_bounds(self) -> OwnedRangeBounds<T> {
            OwnedRangeBounds {
                start: Bound::Unbounded,
                end: Bound::Unbounded,
            }
        }
    }
    #[test]
    fn range_full() {
        let r = ..;
        assert_eq!(
            OwnedRangeBounds::<u32> {
                start: cloned(r.start_bound()),
                end: cloned(r.end_bound()),
            },
            r.into_bounds(),
        );
    }
    impl<T> RangeBoundsExt<T> for RangeInclusive<T> {
        fn into_bounds(self) -> OwnedRangeBounds<T> {
            let (start, end) = self.into_inner();
            OwnedRangeBounds {
                start: Bound::Included(start),
                end: Bound::Included(end),
            }
        }
    }
    #[test]
    fn range_inclusive() {
        let r = 3..=5;
        assert_eq!(
            OwnedRangeBounds::<u32> {
                start: cloned(r.start_bound()),
                end: cloned(r.end_bound()),
            },
            r.into_bounds(),
        );
    }
    impl<T> RangeBoundsExt<T> for RangeTo<T> {
        fn into_bounds(self) -> OwnedRangeBounds<T> {
            OwnedRangeBounds {
                start: Bound::Unbounded,
                end: Bound::Excluded(self.end),
            }
        }
    }
    #[test]
    fn range_to() {
        let r = ..5;
        assert_eq!(
            OwnedRangeBounds::<u32> {
                start: cloned(r.start_bound()),
                end: cloned(r.end_bound()),
            },
            r.into_bounds(),
        );
    }
    impl<T> RangeBoundsExt<T> for RangeToInclusive<T> {
        fn into_bounds(self) -> OwnedRangeBounds<T> {
            OwnedRangeBounds {
                start: Bound::Unbounded,
                end: Bound::Included(self.end),
            }
        }
    }
    #[test]
    fn range_to_inclusive() {
        let r = ..=5;
        assert_eq!(
            OwnedRangeBounds::<u32> {
                start: cloned(r.start_bound()),
                end: cloned(r.end_bound()),
            },
            r.into_bounds(),
        );
    }
}
