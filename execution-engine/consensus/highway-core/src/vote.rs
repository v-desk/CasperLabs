use crate::{
    traits::{Context, HashT},
    validators::ValidatorIndex,
};

/// The observed behavior of a validator at some point in time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Observation<VH: HashT> {
    /// No vote by that validator was observed yet.
    None,
    /// The validator's latest vote.
    Correct(VH),
    /// The validator has been seen
    Faulty,
}

/// The observed behavior of all validators at some point in time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Panorama<VH: HashT>(pub Vec<Observation<VH>>);

/// A vote sent to or received from the network.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vote<C: Context> {
    pub panorama: Panorama<C::VoteHash>,
    // Omitted: Signature, etc.
    pub seq_number: u64,
    pub sender_idx: ValidatorIndex,
    /// The block this is a vote for. Either it or its parent must be the fork choice.
    pub block: C::VoteHash,
}
