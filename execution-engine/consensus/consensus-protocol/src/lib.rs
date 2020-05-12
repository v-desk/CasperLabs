use std::hash::Hash;

mod protocol_state;
mod synchronizer;

#[derive(Debug, PartialEq, Eq)]
pub struct TimerId(u64);

pub trait ConsensusContext {
    /// Consensus specific message.
    /// What gets sent over the wire is opaque to the networking layer,
    /// it is materialized to concrete type in the consensus protocol layer.
    ///
    /// Example ADT might be:
    /// enum Message {
    ///   NewVote(…),
    ///   NewBlock(…),
    ///   RequestDependency(…),
    /// }
    ///
    /// Note that some consensus protocols (like HoneyBadgerBFT) don't have dependencies,
    /// so it's not possible to differentiate between new message and dependency requests
    /// in consensus-agnostic layers.
    type IncomingMessage;

    /// A message that an instance of consensus protocol will create when
    /// it wants to participate in the consensus.
    type OutgoingMessage;

    type ConsensusValue: Hash + PartialEq + Eq;
}

#[derive(Debug)]
pub enum ConsensusProtocolResult<Ctx: ConsensusContext> {
    CreatedNewMessage(Ctx::OutgoingMessage),
    InvalidIncomingMessage(Ctx::IncomingMessage, anyhow::Error),
}

/// An API for a single instance of the consensus.
pub trait ConsensusProtocol<Ctx: ConsensusContext> {
    /// Handle an incoming message (like NewVote, RequestDependency).
    fn handle_message(
        &self,
        msg: Ctx::IncomingMessage,
    ) -> Result<ConsensusProtocolResult<Ctx>, anyhow::Error>;

    /// Triggers consensus to create a new message.
    fn handle_timer(&self, timer_id: TimerId) -> Result<Ctx::OutgoingMessage, anyhow::Error>;
}
