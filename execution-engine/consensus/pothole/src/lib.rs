mod chain;

pub use chain::BlockIndex;
use chain::Chain;
use std::time::{Duration, Instant};

pub trait Block: Clone {}

pub type TimerId = u64;

pub enum Effect<B> {
    ScheduleTimer(TimerId, Instant),
    RequestBlock,
    FinalizedBlock(BlockIndex, B),
}

pub struct Pothole<B: Block> {
    dictator: bool,
    chain: Chain<B>,
    block_timer: Option<TimerId>,
}

const BLOCK_PROPOSE_DURATION: Duration = Duration::from_millis(10_000);

impl<B: Block> Pothole<B> {
    pub fn new(dictator: bool) -> (Self, Vec<Effect<B>>) {
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

    pub fn propose_block(&mut self, block: B) -> Vec<Effect<B>> {
        if self.dictator {
            let index = self.chain.append(block.clone());
            vec![Effect::FinalizedBlock(index, block)]
        } else {
            Vec::new()
        }
    }

    pub fn handle_new_block(&mut self, index: BlockIndex, block: B) -> Vec<Effect<B>> {
        if self.dictator {
            Vec::new()
        } else {
            if self.chain.insert(index, block.clone()).is_none() {
                vec![Effect::FinalizedBlock(index, block)]
            } else {
                vec![]
            }
        }
    }

    pub fn num_blocks(&self) -> usize {
        self.chain.num_blocks()
    }

    pub fn get_last_block(&self) -> Option<&B> {
        self.chain.get_last_block()
    }

    pub fn get_block(&self, index: BlockIndex) -> Option<&B> {
        self.chain.get_block(index)
    }

    pub fn blocks_iterator(&self) -> impl Iterator<Item = (&BlockIndex, &B)> {
        self.chain.blocks_iterator()
    }
}
