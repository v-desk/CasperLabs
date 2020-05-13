use std::hash::Hash;

mod protocol_state;
mod synchronizer;

#[derive(Debug, PartialEq, Eq)]
pub struct TimerId(u64);

#[derive(Debug, PartialEq, Eq)]
pub struct NodeId(u64);

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
    fn handle_timer(
        &self,
        timer_id: TimerId,
    ) -> Result<ConsensusProtocolResult<Ctx>, anyhow::Error>;
}

#[cfg(test)]
mod example {
    use crate::{
        protocol_state::{ProtocolState, Vertex},
        synchronizer::DagSynchronizerState,
        ConsensusContext, ConsensusProtocol, ConsensusProtocolResult, TimerId,
    };
    use anyhow::Error;

    struct HighwayContext();

    #[derive(Debug, Hash, PartialEq, Eq, Clone)]
    struct VIdU64(u64);

    #[derive(Debug, Hash, PartialEq, Eq, Clone)]
    struct DummyVertex {
        id: u64,
        deploy_hash: DeployHash,
    }

    impl Vertex<DeployHash, VIdU64> for DummyVertex {
        fn id(&self) -> VIdU64 {
            VIdU64(self.id)
        }

        fn values(&self) -> Vec<DeployHash> {
            vec![self.deploy_hash.clone()]
        }
    }

    #[derive(Debug, Hash, PartialEq, Eq, Clone)]
    struct DeployHash(u64);

    impl ConsensusContext for HighwayContext {
        type IncomingMessage = HighwayIncomingMessage;
        type OutgoingMessage = HighwayOutgoingMessage;
        type ConsensusValue = DeployHash;
    }

    enum HighwayIncomingMessage {
        NewVertex(DummyVertex),
        RequestVertex(VIdU64),
    }

    enum HighwayOutgoingMessage {}

    impl<P: ProtocolState<VertexId = VIdU64, Vertex = DummyVertex>>
        ConsensusProtocol<HighwayContext>
        for DagSynchronizerState<VIdU64, DummyVertex, DeployHash, P>
    {
        fn handle_message(
            &self,
            msg: <HighwayContext as ConsensusContext>::IncomingMessage,
        ) -> Result<ConsensusProtocolResult<HighwayContext>, Error> {
            match msg {
                HighwayIncomingMessage::RequestVertex(v_id) => unimplemented!(),
                HighwayIncomingMessage::NewVertex(vertex) => unimplemented!(),
            }
        }

        fn handle_timer(
            &self,
            timer_id: TimerId,
        ) -> Result<ConsensusProtocolResult<HighwayContext>, Error> {
            unimplemented!()
        }
    }

    #[test]
    fn foo() {
        assert!(true)
    }
}
