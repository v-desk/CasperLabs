use crate::traits::Context;

/// A block: Chains of blocks are the consensus values in the CBC Casper sense.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Block<C: Context> {
    /// The hash of the block's parent, or `None` for height-0 blocks.
    pub parent: Option<C::VoteHash>,
    /// The total number of ancestors, i.e. the height in the blockchain.
    pub height: u64,
    /// The payload, e.g. a list of transactions.
    pub value: C::ConsensusValue,
}
