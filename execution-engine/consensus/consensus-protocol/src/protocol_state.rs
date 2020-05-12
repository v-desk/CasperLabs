use std::hash::Hash;

pub(crate) trait VertexId {}

pub(crate) trait Vertex<C, Id> {
    fn id(&self) -> Id;

    fn values(&self) -> Vec<C>;
}

pub(crate) trait ProtocolState {
    type VertexId;
    type Vertex;

    type Error;

    fn add_vertex(&mut self, v: Self::Vertex) -> Result<Option<Self::VertexId>, Self::Error>;

    fn get_vertex(&self, v: Self::VertexId) -> Result<Option<Self::Vertex>, Self::Error>;
}
