use std::{collections::BTreeMap, iter};

use crate::{
    state::{State, Weight},
    traits::Context,
    validators::ValidatorIndex,
    vote::{Panorama, Vote},
};

/// A list containing the earliest level-n messages of each member of some committee, for some n.
#[derive(Debug)]
struct Section<'a, C: Context> {
    /// Assigns to each member of a committee the sequence number of the earliest message that
    /// qualifies them for that committee.
    sequence_numbers: BTreeMap<ValidatorIndex, u64>,
    /// A reference to the protocol state this section belongs to.
    state: &'a State<C>,
}

impl<'a, C: Context> Section<'a, C> {
    /// Creates a section assigning to each validator their level-0 vote, i.e. the oldest vote in
    /// their current streak of votes for `candidate` (and descendants), or `None` if their latest
    /// vote is not for `bhash`.
    fn level0(candidate: &'a C::VoteHash, state: &'a State<C>) -> Self {
        let height = state.block(candidate).height;
        let to_lvl0vote = |(idx, vhash): (ValidatorIndex, &'a C::VoteHash)| {
            state
                .swimlane(vhash)
                .take_while(|(_, vote)| state.find_ancestor(&vote.block, height) == Some(candidate))
                .last()
                .map(|(_, vote)| (idx, vote.seq_number))
        };
        let correct_votes = state.panorama().enumerate_correct();
        Section {
            sequence_numbers: correct_votes.filter_map(to_lvl0vote).collect(),
            state,
        }
    }

    /// Returns a section `s` of votes each of which can see a quorum of votes in `self` by
    /// validators that are part of `s`.
    fn next(&self, quorum: Weight) -> Option<Self> {
        let committee = self.pruned_committee(quorum);
        if committee.is_empty() {
            None
        } else {
            Some(self.next_from_committee(quorum, &committee))
        }
    }

    /// Returns the greatest committee of validators whose latest votes can see a quorum of votes
    /// by the committee in `self`.
    fn pruned_committee(&self, quorum: Weight) -> Vec<ValidatorIndex> {
        let mut committee: Vec<ValidatorIndex> = Vec::new();
        let mut next_comm: Vec<ValidatorIndex> = self.sequence_numbers.keys().cloned().collect();
        while next_comm.len() != committee.len() {
            committee = next_comm;
            let sees_quorum = |&idx: &ValidatorIndex| {
                let vhash = self.state.panorama().get(idx).correct().unwrap();
                self.seen_weight(self.state.vote(vhash), &committee) >= quorum
            };
            next_comm = committee.iter().cloned().filter(sees_quorum).collect();
        }
        committee
    }

    /// Returns the section containing the earliest vote of each of the `committee` members that
    /// can see a quorum of votes by `committee` members in `self`.
    fn next_from_committee(&self, quorum: Weight, committee: &[ValidatorIndex]) -> Self {
        let find_first_lvl_n = |&idx: &ValidatorIndex| {
            let (_, vote) = self
                .state
                .swimlane(self.state.panorama().get(idx).correct().unwrap())
                .take_while(|(_, vote)| self.seen_weight(vote, &committee) >= quorum)
                .last()
                .unwrap();
            (idx, vote.seq_number)
        };
        Section {
            sequence_numbers: committee.iter().map(find_first_lvl_n).collect(),
            state: self.state,
        }
    }

    /// Returns the total weight of the `committee`'s members whose message in this section is seen
    /// by `vote`.
    fn seen_weight(&self, vote: &Vote<C>, committee: &[ValidatorIndex]) -> Weight {
        let pan = &vote.panorama;
        let to_weight = |&idx: &ValidatorIndex| self.state.weight(idx);
        let is_seen = |&&idx: &&ValidatorIndex| vote.sender == idx || self.can_see(pan, idx);
        committee.iter().filter(is_seen).map(to_weight).sum()
    }

    /// Returns whether `pan` can see `idx`'s vote in `self`.
    fn can_see(&self, pan: &Panorama<C>, idx: ValidatorIndex) -> bool {
        match (pan.get(idx).correct(), self.sequence_numbers.get(&idx)) {
            (Some(vhash), Some(self_sn)) => self.state.vote(vhash).seq_number >= *self_sn,
            (_, _) => false,
        }
    }
}

/// The result of running the finality detector on a protocol state.
#[derive(Debug, Eq, PartialEq)]
pub enum FinalityResult<V: ConsensusValueT> {
    /// No new block has been finalized yet.
    None,
    /// A new block with these consensus values has been finalized.
    Finalized(Vec<V>),
    /// The fault tolerance threshold has been exceeded: The number of observed equivocation
    /// invalidates this finality detector's results.
    FttExceeded,
}

/// An incremental finality detector.
///
/// It reuses information between subsequent calls, so it must always be applied to the same
/// `State` instance.
#[derive(Debug)]
pub struct FinalityDetector<C: Context> {
    /// The most recent known finalized block.
    last_finalized: Option<C::VoteHash>,
    /// The fault tolerance threshold.
    ftt: Weight,
}

impl<C: Context> FinalityDetector<C> {
    /// Returns the next batch of values, if any has been finalized since the last call.
    // TODO: Iterate this and return multiple finalized blocks.
    // TODO: Verify the consensus instance ID?
    pub fn run(&mut self, state: &State<C>) -> Option<Vec<C::ConsensusValue>> {
        let candidate = self.next_candidate(state)?;
        let total_w: Weight = state.weights().iter().cloned().sum();
        let mut target_lvl = 64; // Levels higher than 64 can't have an effect on a u64 FTT.
        while target_lvl > 0 {
            let lvl = self.find_summit(target_lvl, total_w, candidate, state);
            if lvl == target_lvl {
                self.last_finalized = Some(candidate.clone());
                return Some(state.block(candidate).values.clone());
            }
            target_lvl = lvl;
        }
        None
    }

    /// Returns the number of levels of the highest summit with a quorum that a `target_lvl` summit
    /// would need for the desired FTT. If the returned number is `target_lvl` that means the
    /// `candidate` is finalized. If not, we need to retry with a lower `target_lvl`.
    fn find_summit(
        &self,
        target_lvl: usize,
        total_w: Weight,
        candidate: &C::VoteHash,
        state: &State<C>,
    ) -> usize {
        let quorum = self.quorum_for_lvl(target_lvl, total_w);
        let sec0 = Section::level0(candidate, &state);
        let sections_iter = iter::successors(Some(sec0), |sec| sec.next(quorum));
        sections_iter.take(target_lvl).count()
    }

    /// Returns the quorum required by a summit with the specified level and the required FTT.
    fn quorum_for_lvl(&self, lvl: usize, total_w: Weight) -> Weight {
        // A level-k summit with quorum 50% + t has relative FTT 2t(1 − 1/2^k).
        // We add 1 before every division to round up.
        // TODO: Round properly.
        let ftt128 = self.ftt.0 as u128;
        let t = (1 + ftt128 * (1u128 << (lvl - 1))) / ((1u128 << lvl) - 1);
        Weight((1 + total_w.0) / 2 + t as u64)
    }

    /// Returns the next candidate for finalization, i.e. the lowest block in the fork choice that
    /// has not been finalized yet.
    fn next_candidate<'a>(&self, state: &'a State<C>) -> Option<&'a C::VoteHash> {
        let fork_choice = state.fork_choice(state.panorama())?;
        state.find_ancestor(fork_choice, self.next_height(state))
    }

    /// Returns the height of the next block that will be finalized.
    fn next_height(&self, state: &State<C>) -> u64 {
        let height_plus_1 = |bhash| state.block(bhash).height + 1;
        self.last_finalized.as_ref().map_or(0, height_plus_1)
    }
}
