use crate::{state::State, traits::Context};

/// An incremental finality detector.
///
/// It reuses information between subsequent calls, so it must always be applied to the same
/// `State` instance.
#[derive(Default)]
pub struct FinalityDetector<C: Context> {
    /// The most recent known finalized block.
    last_finalized: Option<C::VoteHash>,
}

impl<C: Context> FinalityDetector<C> {
    /// Returns a list of values that have been finalized since the last call.
    pub fn run(&mut self, state: &State<C>) -> Vec<&C::ConsensusValue> {
        // TODO: Verify the consensus instance ID?
        todo!("{:?}, {:?}", self.last_finalized, state)
    }
}
