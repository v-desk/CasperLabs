use crate::{state::State, traits::Context, validators::ValidatorIndex, vertex::Vertex};

/// An action taken by a validator.
pub enum Effect<C: Context> {
    /// Newly vertex that should be gossiped to peers and added to the protocol state.
    NewVertex(Vertex<C>),
    /// `step` needs to be called at the specified instant.
    ScheduleTimer(u64),
    /// `propose` needs to be called with a value for a new block with the specified instant.
    // TODO: Add more information required by the deploy buffer.
    RequestNewBlock(u64),
}

/// A validator that actively participates in consensus by creating new vertices.
pub struct ActiveValidator<C: Context> {
    /// Our own validator index.
    vidx: ValidatorIndex,
    /// The validator's secret signing key.
    secret: C::ValidatorSecret,
    /// The round exponent: Our subjective rounds are `1 << round_exp` milliseconds long.
    round_exp: u8,
}

impl<C: Context> ActiveValidator<C> {
    /// Returns actions a validator needs to take at the specified `instant`, with the given
    /// protocol `state`.
    pub fn step(&self, state: &State<C>, instant: u64) -> Vec<Effect<C>> {
        let round_len = 1u64 << self.round_exp;
        let round_offset = instant % round_len;
        let round_id = instant - round_offset;
        if round_offset == 0 && state.leader(round_id) == self.vidx {
            vec![Effect::RequestNewBlock(instant)]
        // TODO: We need HWY-55 first, to be able to create votes with correct hash.
        // } else if round_offset * 3 == round_len * 2 {
        //     let panorama = state.panorama().clone();
        //     let prev_hash = panorama.get(self.vidx).correct().unwrap();
        //     let seq_number = state.vote(prev_hash).seq_number + 1;
        //     let witness_vote = WireVote {
        //         hash: todo!(),
        //         panorama,
        //         sender: self.vidx,
        //         values: None,
        //         seq_number,
        //         instant,
        //     };
        //     vec![Effect::NewVertex(Vertex::Vote(witness_vote))]
        } else {
            vec![]
        }
    }

    pub fn on_new_vote(&self, vhash: &C::Hash, state: &State<C>, instant: u64) -> Vec<Effect<C>> {
        todo!("{:?}, {:?}, {:?}", vhash, state, instant)
    }

    /// Propose a new block with the given parent and consensus value.
    pub fn propose(&self, state: &State<C>, values: Vec<C::ConsensusValue>) -> Vec<Effect<C>> {
        todo!("{:?}, {:?}, {:?}", state, values, self.secret)
        // vec![Effect::NewVertex(Vertex::Vote(vote))]
    }
}
