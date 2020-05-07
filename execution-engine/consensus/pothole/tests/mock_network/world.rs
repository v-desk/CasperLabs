use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::mem;
use std::rc::Rc;
use std::time::{Duration, Instant};

use pothole::TimerId;

use super::{NetworkMessage, NodeId};

pub struct MsgQueueEntry {
    pub sender: NodeId,
    pub message: NetworkMessage,
}

pub struct World {
    current_time: Instant,
    message_queue: HashMap<NodeId, VecDeque<MsgQueueEntry>>,
    timers: HashMap<NodeId, BTreeMap<Instant, TimerId>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            current_time: Instant::now(),
            message_queue: Default::default(),
            timers: Default::default(),
        }
    }

    pub fn send_message(&mut self, sender: NodeId, recipient: NodeId, message: NetworkMessage) {
        self.message_queue
            .entry(recipient)
            .or_insert_with(Default::default)
            .push_back(MsgQueueEntry { sender, message });
    }

    pub fn recv_message(&mut self, recipient: NodeId) -> Option<MsgQueueEntry> {
        self.message_queue
            .get_mut(&recipient)
            .and_then(|queue| queue.pop_front())
    }

    pub fn advance_time(&mut self, duration: Duration) {
        self.current_time += duration;
    }

    pub fn schedule_timer(&mut self, node: NodeId, timer: TimerId, instant: Instant) {
        self.timers
            .entry(node)
            .or_insert_with(Default::default)
            .insert(instant, timer);
    }

    pub fn fire_timers(&mut self, node: NodeId) -> Vec<TimerId> {
        let timers_ref = self.timers.entry(node).or_insert_with(Default::default);
        let timers_to_remain =
            // adding 1 ms so that timers scheduled for now will also fire
            timers_ref.split_off(&(self.current_time + Duration::from_millis(1)));
        mem::replace(timers_ref, timers_to_remain)
            .into_iter()
            .map(|(_, val)| val)
            .collect()
    }

    pub fn is_queue_empty(&self) -> bool {
        self.message_queue.iter().all(|(_, queue)| queue.is_empty())
    }

    pub fn time_to_earliest_timer(&self) -> Option<Duration> {
        self.timers
            .iter()
            .filter_map(|(_, timers)| timers.keys().next())
            .min()
            .map(|instant| instant.saturating_duration_since(self.current_time))
    }
}

pub struct WorldHandle {
    world: Rc<RefCell<World>>,
    node_id: NodeId,
}

impl WorldHandle {
    pub fn new(world: Rc<RefCell<World>>, node_id: NodeId) -> Self {
        Self { world, node_id }
    }

    pub fn send_message(&self, dst: NodeId, msg: NetworkMessage) {
        self.world.borrow_mut().send_message(self.node_id, dst, msg);
    }

    pub fn recv_message(&self) -> Option<MsgQueueEntry> {
        self.world.borrow_mut().recv_message(self.node_id)
    }

    pub fn schedule_timer(&self, timer: TimerId, instant: Instant) {
        self.world
            .borrow_mut()
            .schedule_timer(self.node_id, timer, instant);
    }

    pub fn fire_timers(&self) -> Vec<TimerId> {
        self.world.borrow_mut().fire_timers(self.node_id)
    }
}

mod tests {
    use super::World;
    use std::time::{Duration, Instant};

    #[test]
    fn test_timers() {
        let mut world = World::new();
        let instant = Instant::now();

        let one_second = Duration::from_millis(1000);
        let two_seconds = Duration::from_millis(2000);

        let node_id = "TestNode";

        assert!(world.fire_timers(node_id).is_empty());

        world.schedule_timer(node_id, 0, instant + two_seconds);

        assert!(world.fire_timers(node_id).is_empty());

        world.advance_time(one_second);

        assert!(world.fire_timers(node_id).is_empty());

        world.advance_time(one_second);

        assert!(world.fire_timers(node_id).len() == 1);
    }
}
