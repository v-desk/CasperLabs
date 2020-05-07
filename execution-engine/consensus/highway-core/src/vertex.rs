use crate::{evidence::Evidence, traits::Context, validators::ValidatorIndex, vote::Panorama};

/// A dependency of a `Vertex` that can be satisfied by one or more other vertices.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Dependency<C: Context> {
    Vote(C::VoteHash),
    Evidence(ValidatorIndex),
}

/// An element of the protocol state, that might depend on other elements.
///
/// It is the vertex in a directed acyclic graph, whose edges are dependencies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vertex<C: Context> {
    Vote(WireVote<C>),
    Evidence(Evidence<C>),
}

/// A vote as it is sent over the wire, possibly containing a new block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WireVote<C: Context> {
    pub hash: C::VoteHash,
    pub panorama: Panorama<C::VoteHash>,
    pub sender: C::ValidatorId,
    pub value: Option<C::ConsensusValue>,
    pub seq_number: u64,
}
