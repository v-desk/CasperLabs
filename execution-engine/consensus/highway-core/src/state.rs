use std::{collections::HashMap, time::Duration};

use crate::{
    block::Block,
    evidence::Evidence,
    traits::Context,
    validators::{ValidatorIndex, Validators},
    vertex::{Dependency, Vertex, WireVote},
    vote::{Observation, Panorama, Vote},
};

/// The result of trying to add a vertex to the protocol state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AddVertexOutcome<C: Context> {
    /// The vertex was successfully added.
    Success,
    /// The vertex could not be added because it is missing a dependency. The vertex itself is
    /// returned, together with the missing dependency.
    MissingDependency(Vertex<C>, Dependency<C>),
    /// The vertex is invalid and cannot be added to the protocol state at all.
    // TODO: Distinction — is it the vertex creator's attributable fault?
    Invalid(Vertex<C>),
}

#[derive(Debug)]
pub struct StateParams<C: Context> {
    /// The protocol instance ID. This needs to be unique, to prevent replay attacks.
    instance_id: C::InstanceId,
    /// The validator IDs and weight map.
    validators: Validators<C::ValidatorId>,
    /// The duration of a single tick.
    tick_length: Duration,
}

/// A passive instance of the Highway protocol, containing its local state.
///
/// Both observers and active validators must instantiate this, pass in all incoming vertices from
/// peers, and use a [FinalityDetector](../finality_detector/struct.FinalityDetector.html) to
/// determine the outcome of the consensus process.
#[derive(Debug)]
pub struct State<C: Context> {
    /// The parameters that remain constant for the duration of this consensus instance.
    params: StateParams<C>,
    /// All votes imported so far, by hash.
    // TODO: HashMaps prevent deterministic tests.
    votes: HashMap<C::VoteHash, Vote<C>>,
    /// All blocks, by hash.
    blocks: HashMap<C::VoteHash, Block<C>>,
    /// Evidence to prove a validator malicious, by index.
    evidence: HashMap<ValidatorIndex, Evidence<C>>,
}

impl<C: Context> State<C> {
    /// Try to add an incoming vertex to the protocol state.
    ///
    /// If the vertex is invalid, or if there are dependencies that need to be added first, returns
    /// `Invalid` resp. `MissingDependency`.
    pub fn add_vertex(&mut self, vertex: Vertex<C>) -> AddVertexOutcome<C> {
        match vertex {
            Vertex::Vote(vote) => self.add_vote(vote),
            Vertex::Evidence(evidence) => self.add_evidence(evidence),
        }
    }

    /// Returns a vertex that satisfies the dependency, if available.
    ///
    /// If we send a vertex to a peer who is missing a dependency, they will ask us for it. In that
    /// case, `get_dependency` will always return `Some`, unless the peer is faulty.
    pub fn get_dependency(&self, dependency: Dependency<C>) -> Option<Vertex<C>> {
        match dependency {
            Dependency::Evidence(idx) => self.evidence.get(&idx).cloned().map(Vertex::Evidence),
            Dependency::Vote(hash) => self.wire_vote(hash).map(Vertex::Vote),
        }
    }

    fn wire_vote(&self, hash: C::VoteHash) -> Option<WireVote<C>> {
        let vote = self.votes.get(&hash)?.clone();
        let values = self.blocks.get(&hash).map(|block| block.values.clone());
        Some(WireVote {
            hash,
            panorama: vote.panorama.clone(),
            sender: self.params.validators.id_of(vote.sender_idx).clone(),
            values,
            seq_number: vote.seq_number,
        })
    }

    fn missing_dependency(&self, vote: &WireVote<C>) -> Option<Dependency<C>> {
        for (idx, observation) in vote.panorama.0.iter().enumerate() {
            match observation {
                Observation::Faulty if !self.evidence.contains_key(&idx.into()) => {
                    return Some(Dependency::Evidence(idx.into()));
                }
                Observation::Correct(hash) if !self.votes.contains_key(hash) => {
                    return Some(Dependency::Vote(hash.clone()));
                }
                _ => (),
            }
        }
        None
    }

    fn add_vote(&mut self, wvote: WireVote<C>) -> AddVertexOutcome<C> {
        if let Some(dep) = self.missing_dependency(&wvote) {
            return AddVertexOutcome::MissingDependency(Vertex::Vote(wvote), dep);
        }
        let hash = wvote.hash.clone();
        let fork_choice: Option<C::VoteHash> = self.fork_choice(&wvote.panorama);
        let block = if let Some(values) = wvote.values {
            let height = fork_choice
                .as_ref()
                .map_or(0, |hash| self.blocks[hash].height + 1);
            let block = Block {
                parent: fork_choice,
                height,
                values,
            };
            self.blocks.insert(hash.clone(), block);
            hash.clone()
        } else {
            // If the vote didn't introduce a new block, it votes for the fork choice itself.
            fork_choice.unwrap()
        };
        // TODO: Validation; e.g. invalid sender.
        let sender_idx = self.params.validators.index_of(&wvote.sender).unwrap();
        let vote = Vote {
            panorama: wvote.panorama,
            seq_number: wvote.seq_number,
            sender_idx,
            block,
        };
        self.votes.insert(hash, vote);
        AddVertexOutcome::Success
    }

    fn add_evidence(&mut self, evidence: Evidence<C>) -> AddVertexOutcome<C> {
        if let Some(idx) = self.params.validators.index_of(evidence.perpetrator()) {
            self.evidence.insert(idx, evidence);
        } else {
            return AddVertexOutcome::Invalid(Vertex::Evidence(evidence));
        }
        AddVertexOutcome::Success
    }

    fn fork_choice(&self, panorama: &Panorama<C::VoteHash>) -> Option<C::VoteHash> {
        todo!("{:?}", panorama)
    }
}
