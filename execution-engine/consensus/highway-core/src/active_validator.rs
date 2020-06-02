use std::time::Instant;

use crate::{state::State, traits::Context, vertex::Vertex};

/// An action taken by a validator.
pub enum Effect<C: Context> {
    /// Newly vertex that should be gossiped to peers and added to the protocol state.
    NewVertex(Vertex<C>),
    /// `step` needs to be called at this time.
    ScheduleTimer(Instant),
    /// `propose` needs to be called with a value for a new block with the specified parent.
    RequestNewBlock(Option<C::Hash>),
}

/// A validator that actively participates in consensus by creating new vertices.
pub struct ActiveValidator<C: Context> {
    /// The validator's secret signing key.
    secret: C::ValidatorSecret,
}

impl<C: Context> ActiveValidator<C> {
    /// Returns actions a validator needs to take at the specified `time`, with the given protocol
    /// `state`.
    pub fn step(&self, _state: &State<C>, _time: Instant) -> Vec<Effect<C>> {
        todo!("{:?}", self.secret)
    }

    /// Propose a new block with the given parent and consensus value.
    pub fn propose(&self, state: &State<C>, values: Vec<C::ConsensusValue>) -> Vec<Effect<C>> {
        todo!("{:?}, {:?}", state, values)
        // vec![Effect::NewVertex(Vertex::Vote(vote))]
    }
}
