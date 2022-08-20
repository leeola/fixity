// pub mod gcounter;
pub mod appendlog;
pub mod prolly_tree;
pub mod ptr;
/*
pub mod vclock {
    use crate::gcounter::GCounter;
    use std::{
        cmp::{Ordering, PartialEq, PartialOrd},
        ops::{Deref, DerefMut},
    };
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(
        feature = "rkyv",
        derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
    )]
    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    pub struct VClock(GCounter);
    impl VClock {
        pub fn new() -> Self {
            Self::default()
        }
    }
    impl Deref for VClock {
        type Target = GCounter;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl DerefMut for VClock {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl PartialOrd for VClock {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            // NIT: This func is rather expensive :/
            use Ordering::{Equal, Greater, Less};
            // Avoiding full map alloc by iter instead of merging
            let iter = self.iter().map(|(rid, lhs)| (lhs, other.get(rid))).chain(
                // if we exhaust items in Self and haven't returned None (Concurrent)
                // yet, we chain the remaining items.
                other
                    .iter()
                    .filter(|(rid, _)| self.contains_rid(rid))
                    .map(|(_, rhs)| (&0, Some(rhs))),
            );
            // PERF: This could be implemented as a TryFold with .ok() as well.
            // The early return as an escape hatch might be less work than Err()
            // for the compiler though, hard to say, worth checking in the future.
            let mut ord = Equal;
            for (lhs, rhs) in iter {
                ord = match (ord, lhs.cmp(rhs.unwrap_or(&0))) {
                    (Equal, Equal) => Equal,
                    (Equal, Greater) | (Greater, Equal) | (Greater, Greater) => Greater,
                    (Equal, Less) | (Less, Equal) | (Less, Less) => Less,
                    (Greater, Less) | (Less, Greater) => return None,
                }
            }
            Some(ord)
        }
    }
}
pub mod pncounter {
    use crate::gcounter::{GCounter, ReplicaId};
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(
        feature = "rkyv",
        derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
    )]
    #[derive(Debug, Default)]
    pub struct PNCounter {
        inc: GCounter,
        dec: GCounter,
    }
    impl PNCounter {
        pub fn new() -> Self {
            Self::default()
        }
        pub fn inc(&mut self, rid: ReplicaId) {
            self.inc.inc(rid)
        }
        pub fn dec(&mut self, rid: ReplicaId) {
            self.dec.inc(rid)
        }
        pub fn value(&self) -> u32 {
            self.inc.value() - self.dec.value()
        }
        pub fn merge(&mut self, other: &Self) {
            self.inc.merge(&other.inc);
            self.dec.merge(&other.dec);
        }
    }
    #[cfg(test)]
    pub mod test {
        use super::*;
        #[test]
        fn poc() {
            let mut a = PNCounter::default();
            a.inc(0);
            assert_eq!(a.value(), 1);
            a.inc(1);
            a.inc(0);
            assert_eq!(a.value(), 3);
        }
    }
}
*/
