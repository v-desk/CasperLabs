use crate::{traits::Context, vertex::WireVote};

/// Evidence that a validator is faulty.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Evidence<C: Context> {
    /// The validator produced two votes with the same sequence number.
    Equivocation(WireVote<C>, WireVote<C>),
}

impl<C: Context> Evidence<C> {
    // TODO: Verify whether the evidence is conclusive. Or as part of deserialization?

    /// Returns the ID of the faulty validator.
    pub(crate) fn perpetrator(&self) -> &C::ValidatorId {
        match self {
            Evidence::Equivocation(vote0, _) => &vote0.sender,
        }
    }
}
