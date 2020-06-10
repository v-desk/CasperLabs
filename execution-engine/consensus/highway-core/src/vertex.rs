use std::iter;

use serde::{Deserialize, Serialize};

use crate::{
    evidence::Evidence,
    traits::{Context, ValidatorSecret},
    validators::ValidatorIndex,
    vote::Panorama,
};

/// A dependency of a `Vertex` that can be satisfied by one or more other vertices.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Dependency<C: Context> {
    Vote(C::Hash),
    Evidence(ValidatorIndex),
}

/// An element of the protocol state, that might depend on other elements.
///
/// It is the vertex in a directed acyclic graph, whose edges are dependencies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vertex<C: Context> {
    Vote(SignedWireVote<C>),
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
            Vertex::Vote(swvote) => Box::new(swvote.wire_vote.values.iter().flat_map(|v| v.iter())),
            Vertex::Evidence(_) => Box::new(iter::empty()),
        }
    }
}
/// A vote as it is sent over the wire, possibly containing a new block.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SignedWireVote<C: Context> {
    pub wire_vote: WireVote<C>,
    pub signature: <C::ValidatorSecret as ValidatorSecret>::Signature,
}

impl<C: Context> SignedWireVote<C> {
    pub fn new(wire_vote: WireVote<C>, secret_key: &C::ValidatorSecret) -> Self {
        let signature = secret_key.sign(&wire_vote.hash());
        SignedWireVote {
            wire_vote,
            signature,
        }
    }

    pub fn hash(&self) -> C::Hash {
        self.wire_vote.hash()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "C::Hash: Serialize",
    deserialize = "C::Hash: Deserialize<'de>",
))]
pub struct WireVote<C: Context> {
    pub panorama: Panorama<C>,
    pub sender: ValidatorIndex,
    pub values: Option<Vec<C::ConsensusValue>>,
    pub seq_number: u64,
    pub instant: u64,
}

impl<C: Context> WireVote<C> {
    /// Returns the vote's hash, which is used as a vote identifier.
    // TODO: This involves serializing and hashing. Memoize?
    pub fn hash(&self) -> C::Hash {
        // TODO: Use serialize_into to avoid allocation?
        C::hash(&bincode::serialize(self).expect("serialize WireVote"))
    }
}
