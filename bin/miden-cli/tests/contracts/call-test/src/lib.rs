#![no_std]
#![feature(alloc_error_handler)]

use miden::{
    Felt,
    StorageMap,
    StorageMapAccess,
    Value,
    ValueAccess,
    Word,
    component,
    export_type,
    felt,
};

/// A 2D point. Encoded as 2 Felts: [x, y]
#[export_type]
pub struct Point {
    pub x: Felt,
    pub y: Felt,
}

#[component]
struct CallTest {
    /// A single value slot for testing state deltas
    #[storage(description = "a stored value")]
    stored_value: Value,

    /// A storage map for key-value testing
    #[storage(description = "a storage map")]
    data: StorageMap,
}

#[component]
impl CallTest {
    /// Adds two felts. Pure, no state change.
    pub fn add(&self, a: Felt, b: Felt) -> Felt {
        a + b
    }

    /// Multiplies two felts. Pure, no state change.
    pub fn mul(&self, a: Felt, b: Felt) -> Felt {
        a * b
    }

    /// Adds two points component-wise.
    pub fn add_points(&self, a: Point, b: Point) -> Point {
        Point { x: a.x + b.x, y: a.y + b.y }
    }

    /// Returns the sum of a point's components.
    pub fn point_sum(&self, p: Point) -> Felt {
        p.x + p.y
    }

    /// Sets the stored value and returns the old one.
    pub fn set_value(&mut self, value: Word) -> Word {
        self.stored_value.write(value)
    }

    /// Reads the stored value.
    pub fn get_value(&self) -> Word {
        self.stored_value.read()
    }

    /// Increments a counter in the storage map.
    pub fn increment(&mut self) -> Felt {
        let key = Word::from_u64_unchecked(0, 0, 0, 1);
        let current: Felt = self.data.get(&key);
        let new_val = current + felt!(1);
        self.data.set(key, new_val);
        new_val
    }

    /// Reads the counter from the storage map.
    pub fn get_counter(&self) -> Felt {
        let key = Word::from_u64_unchecked(0, 0, 0, 1);
        self.data.get(&key)
    }
}
