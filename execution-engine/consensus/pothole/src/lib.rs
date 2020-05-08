mod chain;

pub use chain::BlockIndex;
use chain::Chain;
use std::{
    collections::BTreeSet,
    fmt::Debug,
    time::{Duration, Instant},
};

/// A trait for block types to implement
pub trait Block: Clone + Debug {}
impl<T> Block for T where T: Clone + Debug {}

pub trait NodeId: Clone + Debug + PartialEq + Ord {}
impl<T> NodeId for T where T: Clone + Debug + PartialEq + Ord {}

/// An identifier for a timer set to fire at a later moment
pub type TimerId = u64;

/// Possible effects that could result from the consensus operations
pub enum Effect<B> {
    /// A request for a timer to be scheduled
    ScheduleTimer(TimerId, Instant),
    /// A request for a block to be proposed
    RequestBlock,
    /// A notification that a block has been finalized
    FinalizedBlock(BlockIndex, B),
}

/// The state of the consensus protocol
pub struct Pothole<B: Block> {
    dictator: bool,
    chain: Chain<B>,
    block_timer: Option<TimerId>,
}

const BLOCK_PROPOSE_DURATION: Duration = Duration::from_millis(10_000);

impl<B: Block> Pothole<B> {
    /// Creates a new instance of the protocol. If this node is the first (lexicographically) among
    /// the peers, it becomes the dictator (the node determining the order of blocks). Returns the
    /// protocol instance along with some possible side-effects.
    pub fn new<N: NodeId>(our_id: &N, all_nodes: &BTreeSet<N>) -> (Self, Vec<Effect<B>>) {
        let dictator = Some(our_id) == all_nodes.iter().next();
        (
            Self {
                dictator,
                chain: Default::default(),
                block_timer: if dictator { Some(0) } else { None },
            },
            if dictator {
                vec![Effect::ScheduleTimer(
                    0, // TODO: do timer ids come from outside or do we set them arbitrarily?
                    Instant::now() + BLOCK_PROPOSE_DURATION,
                )]
            } else {
                Vec::new()
            },
        )
    }

    /// Handles a timer event (scheduled according to an earlier ScheduleTimer request).
    pub fn handle_timer(&mut self, timer: TimerId) -> Vec<Effect<B>> {
        if Some(timer) == self.block_timer {
            vec![
                Effect::RequestBlock,
                Effect::ScheduleTimer(timer, Instant::now() + BLOCK_PROPOSE_DURATION),
            ]
        } else {
            Vec::new()
        }
    }

    /// Proposes a new block for the chain.
    pub fn propose_block(&mut self, block: B) -> Vec<Effect<B>> {
        if self.dictator {
            let index = self.chain.append(block.clone());
            vec![Effect::FinalizedBlock(index, block)]
        } else {
            Vec::new()
        }
    }

    /// Handles a notification about a new block having been finalized.
    pub fn handle_new_block(&mut self, index: BlockIndex, block: B) -> Vec<Effect<B>> {
        if self.dictator {
            Vec::new()
        } else if self.chain.insert(index, block.clone()).is_none() {
            vec![Effect::FinalizedBlock(index, block)]
        } else {
            vec![]
        }
    }

    /// Returns the number of blocks in the chain.
    pub fn num_blocks(&self) -> usize {
        self.chain.num_blocks()
    }

    /// Returns the reference to the last block in the chain.
    pub fn get_last_block(&self) -> Option<&B> {
        self.chain.get_last_block()
    }

    /// Gets the block at a given index.
    pub fn get_block(&self, index: BlockIndex) -> Option<&B> {
        self.chain.get_block(index)
    }

    /// Returns an iterator over blocks in the chain.
    pub fn blocks_iterator(&self) -> impl Iterator<Item = (&BlockIndex, &B)> {
        self.chain.blocks_iterator()
    }
}
