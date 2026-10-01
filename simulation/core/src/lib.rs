//! alife-core: deterministic foundations shared by world and minds.

pub mod events;
pub mod fx;
pub mod hash;
pub mod rng;

pub use events::{EventKey, EventQueue, RateSchedule};
pub use fx::{fx, Fx};
pub use hash::{StableHash, StateHasher};
pub use rng::{key4, mix64, stream, Rng};
