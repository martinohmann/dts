//! Types for carrying state across multiple transformations.

use dts_json::Value;
use std::collections::VecDeque;

/// Represents the transform state.
#[derive(Default)]
pub struct State {
    queue: VecDeque<Value>,
}

impl State {
    /// Creates a new `State`.
    pub fn new() -> Self {
        State::default()
    }

    /// Returns a mutable reference to the underlying value queue.
    pub fn queue_mut(&mut self) -> &mut VecDeque<Value> {
        &mut self.queue
    }
}
