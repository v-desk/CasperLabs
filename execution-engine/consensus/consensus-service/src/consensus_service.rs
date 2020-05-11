//! Consensus service is a component that will be communicating with the reactor.
//! It will receive events (like incoming message event or create new message event)
//! and propagate them to the underlying consensus protocol.
//! It tries to know as little as possible about the underlying consensus. The only thing
//! it assumes is the concept of era/epoch and that each era runs separate consensus instance.
//! Most importantly, it doesn't care about what messages it's forwarding.

use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    time::{Duration, Instant},
};

use crate::traits::{Effect, EraId, MessageWireFormat};
use consensus_protocol::{ConsensusContext, ConsensusProtocol, ConsensusProtocolResult};

pub enum ConsensusServiceError {
    InvalidFormat(String),
    InternalError(anyhow::Error),
}

pub enum Event {
    IncomingMessage(MessageWireFormat),
    CreateMessage(EraId),
}

struct EraConfig {
    era_length: Duration,
    //TODO: Are these necessary for every consensus protocol?
    booking_duration: Duration,
    entropy_duration: Duration,
}

struct EraInstance<Id> {
    era_id: Id,
    era_start: Instant,
    era_end: Instant,
}

/// API between the reactor and consensus component.
pub trait ConsensusService {
    fn handle_event(&mut self, event: Event) -> Result<Effect<Event>, ConsensusServiceError>;
}

struct EraSupervisor<C: ConsensusContext> {
    // A map of active consensus protocols.
    // A value is a trait so that we can run different consensus protocol instances per era.
    active_eras: HashMap<EraId, Box<dyn ConsensusProtocol<C>>>,
    era_config: EraConfig,
}

impl<C: ConsensusContext> ConsensusService for EraSupervisor<C>
where
    C::IncomingMessage: TryFrom<MessageWireFormat>,
    C::OutgoingMessage: Into<MessageWireFormat>,
{
    fn handle_event(&mut self, event: Event) -> Result<Effect<Event>, ConsensusServiceError> {
        match event {
            Event::CreateMessage(era_id) => match self.active_eras.get(&era_id) {
                None => todo!("Handle missing eras."),
                Some(consensus) => consensus
                    .create_message()
                    .map(|out_msg| {
                        let wire_msg: MessageWireFormat = out_msg.into();
                        todo!("Create an effect to broadcast new msg")
                    })
                    .map_err(|err| ConsensusServiceError::InternalError(err)),
            },
            Event::IncomingMessage(wire_msg) => match self.active_eras.get(&wire_msg.era_id) {
                None => todo!("Handle missing eras."),
                Some(consensus) => {
                    let message: C::IncomingMessage = wire_msg
                        .try_into()
                        .map_err(|_| ConsensusServiceError::InvalidFormat("".to_string()))?;
                    consensus
                        .handle_message(message)
                        .map(|result| match result {
                            ConsensusProtocolResult::InvalidIncomingMessage(msg, error) => {}
                            ConsensusProtocolResult::CreatedNewMessage(out_msg) => {
                                let wire_msg: MessageWireFormat = out_msg.into();
                                todo!("Create an effect to broadcast new msg")
                            }
                        });
                    Ok(Effect::Nothing)
                }
            },
        }
    }
}
