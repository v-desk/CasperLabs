use crate::{state::State, traits::Context};

/// A block: Chains of blocks are the consensus values in the CBC Casper sense.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Block<C: Context> {
    /// The hash of the block's parent, or `None` for height-0 blocks.
    pub parent: Option<C::VoteHash>,
    /// The total number of ancestors, i.e. the height in the blockchain.
    pub height: u64,
    /// The payload, e.g. a list of transactions.
    pub values: Vec<C::ConsensusValue>,
}

impl<C: Context> Block<C> {
    /// Creates a new block with the given parent and values. Panics if parent does not exist.
    pub fn new(
        parent: Option<C::VoteHash>,
        values: Vec<C::ConsensusValue>,
        state: &State<C>,
    ) -> Block<C> {
        let parent_plus_one = |hash| state.block(hash).height + 1;
        let height = parent.as_ref().map_or(0, parent_plus_one);
        Block {
            parent,
            height,
            values,
        }
    }
}
