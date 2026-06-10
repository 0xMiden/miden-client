//! Shared internal types used by the PSWAP module.

use miden_protocol::Felt;

/// Wrapper over [`Felt`] providing `Ord`/`Hash`-compatible derives for use as
/// a `BTreeMap` key. `Felt` itself doesn't implement these because field
/// elements have multiple canonical representations.
#[derive(Clone, Copy)]
pub(crate) struct OrderIdKey(Felt);

impl From<Felt> for OrderIdKey {
    fn from(value: Felt) -> Self {
        Self(value)
    }
}

impl From<OrderIdKey> for Felt {
    fn from(key: OrderIdKey) -> Self {
        key.0
    }
}

impl PartialEq for OrderIdKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_canonical_u64() == other.0.as_canonical_u64()
    }
}
impl Eq for OrderIdKey {}
impl PartialOrd for OrderIdKey {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for OrderIdKey {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.as_canonical_u64().cmp(&other.0.as_canonical_u64())
    }
}

#[cfg(test)]
mod tests {
    use core::cmp::Ordering;

    use miden_protocol::Felt;

    use super::OrderIdKey;

    /// Equality compares canonical `u64` values, not raw `Felt` reps.
    #[test]
    fn order_id_key_equality_tracks_canonical_value() {
        let a = OrderIdKey::from(Felt::new(42).unwrap());
        let b = OrderIdKey::from(Felt::new(42).unwrap());
        let c = OrderIdKey::from(Felt::new(43).unwrap());
        // `OrderIdKey` intentionally doesn't derive `Debug`; use bool asserts.
        assert!(a == b);
        assert!(a != c);
    }

    /// `cmp` / `partial_cmp` order by canonical value and agree with each other.
    #[test]
    fn order_id_key_ordering_tracks_canonical_value() {
        let small = OrderIdKey::from(Felt::new(1).unwrap());
        let large = OrderIdKey::from(Felt::new(2).unwrap());
        assert_eq!(small.cmp(&large), Ordering::Less);
        assert_eq!(large.cmp(&small), Ordering::Greater);
        assert_eq!(small.partial_cmp(&small), Some(Ordering::Equal));
        assert!(small < large);
    }
}
