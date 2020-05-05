use crate::ConsensusContext;
use std::hash::Hash;

pub(crate) trait VertexId {}

pub(crate) trait Vertex<C, Id> {
    fn id(&self) -> Id;

    fn values(&self) -> &[C];
}

pub(crate) trait ProtocolState<Ctx: ConsensusContext> {
    type VId: VertexId + Hash + PartialEq + Eq;
    type V: Vertex<Ctx::ConsensusValue, Self::VId>;

    type Error;

    fn add_vertex(&mut self, v: Self::V) -> Result<Option<Self::VId>, Self::Error>;

    fn get_vertex(&self, v: Self::VId) -> Result<Option<Self::V>, Self::Error>;
}
