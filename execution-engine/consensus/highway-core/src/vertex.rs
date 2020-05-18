use std::iter;

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

impl<C: Context> Vertex<C> {
    /// Returns an iterator over all consensus values mentioned in this vertex.
    ///
    /// These need to be validated before passing the vertex into the protocol state. E.g. if
    /// `C::ConsensusValue` is a transaction, it should be validated first (correct signature,
    /// structure, gas limit, etc.). If it is a hash of a transaction, the transaction should be
    /// obtained _and_ validated. Only after that, the vertex can be considered valid.
    pub fn values<'a>(&'a self) -> Box<dyn Iterator<Item = &'a C::ConsensusValue> + 'a> {
        match self {
            Vertex::Vote(wvote) => Box::new(wvote.values.iter().flat_map(|v| v.iter())),
            Vertex::Evidence(_) => Box::new(iter::empty()),
        }
    }
}

/// A vote as it is sent over the wire, possibly containing a new block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WireVote<C: Context> {
    pub hash: C::VoteHash,
    pub panorama: Panorama<C>,
    pub sender: ValidatorIndex,
    pub values: Option<Vec<C::ConsensusValue>>,
    pub seq_number: u64,
}
