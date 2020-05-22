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
use consensus_protocol::{ConsensusContext, ConsensusProtocol, ConsensusProtocolResult, TimerId};

pub enum ConsensusServiceError {
    InvalidFormat(String),
    InternalError(anyhow::Error),
}

pub enum Event {
    IncomingMessage(MessageWireFormat),
    Timer(EraId, TimerId),
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
    fn handle_event(&mut self, event: Event) -> Result<Vec<Effect<Event>>, ConsensusServiceError>;
}

struct EraSupervisor<C: ConsensusContext> {
    // A map of active consensus protocols.
    // A value is a trait so that we can run different consensus protocol instances per era.
    active_eras: HashMap<EraId, Box<dyn ConsensusProtocol<C>>>,
    era_config: EraConfig,
}

impl<C: ConsensusContext> ConsensusService for EraSupervisor<C>
where
    C::Message: TryFrom<MessageWireFormat> + Into<MessageWireFormat>,
{
    fn handle_event(&mut self, event: Event) -> Result<Vec<Effect<Event>>, ConsensusServiceError> {
        match event {
            Event::Timer(era_id, timer_id) => match self.active_eras.get_mut(&era_id) {
                None => todo!("Handle missing eras."),
                Some(consensus) => consensus
                    .handle_timer(timer_id)
                    .map(|result_vec| {
                        result_vec
                            .into_iter()
                            .map(|result| match result {
                                ConsensusProtocolResult::InvalidIncomingMessage(_msg, _error) => {
                                    unimplemented!()
                                }
                                ConsensusProtocolResult::CreatedNewMessage(out_msg) => {
                                    let _wire_msg: MessageWireFormat = out_msg.into();
                                    todo!("Create an effect to broadcast new msg")
                                }
                                ConsensusProtocolResult::ScheduleTimer(_delay, _timer_id) => {
                                    unimplemented!()
                                }
                                ConsensusProtocolResult::CreateNewBlock => unimplemented!(),
                            })
                            .collect()
                    })
                    .map_err(ConsensusServiceError::InternalError),
            },
            Event::IncomingMessage(wire_msg) => match self.active_eras.get_mut(&wire_msg.era_id) {
                None => todo!("Handle missing eras."),
                Some(consensus) => {
                    let message: C::Message = wire_msg
                        .try_into()
                        .map_err(|_| ConsensusServiceError::InvalidFormat("".to_string()))?;
                    consensus
                        .handle_message(message)
                        .map(|result_vec| {
                            result_vec
                                .into_iter()
                                .map(|result| match result {
                                    ConsensusProtocolResult::InvalidIncomingMessage(
                                        _msg,
                                        _error,
                                    ) => unimplemented!(),
                                    ConsensusProtocolResult::CreatedNewMessage(out_msg) => {
                                        let _wire_msg: MessageWireFormat = out_msg.into();
                                        todo!("Create an effect to broadcast new msg")
                                    }
                                    ConsensusProtocolResult::ScheduleTimer(_delay, _timer_id) => {
                                        unimplemented!()
                                    }
                                    ConsensusProtocolResult::CreateNewBlock => unimplemented!(),
                                })
                                .collect()
                        })
                        .map_err(ConsensusServiceError::InternalError)
                }
            },
        }
    }
}
