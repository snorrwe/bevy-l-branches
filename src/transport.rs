//! Comm layer between bevy and yew

use bevy::prelude::*;

pub use async_std::channel::{Receiver, Sender};

#[derive(Clone, Resource)]
pub struct EventHandle {
    pub receiver: Receiver<Event>,
    pub sender: Sender<Event>,
}

impl EventHandle {
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = async_std::channel::bounded(capacity);
        Self { receiver, sender }
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    AddTopic,
}

pub struct EventPlugin {
    pub handle: EventHandle,
}

pub struct SendEvent(pub Event);
pub struct ReceiveEvent(pub Event);

impl Plugin for EventPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(self.handle.clone())
            .add_event::<SendEvent>()
            .add_event::<ReceiveEvent>()
            .add_system(receive)
            .add_system(send);
    }
}

fn receive(handle: ResMut<EventHandle>, mut events: EventWriter<ReceiveEvent>) {
    if let Ok(ev) = handle.receiver.try_recv() {
        debug!("message from UI: {ev:?}");
        events.send(ReceiveEvent(ev));
    }
}

fn send(handle: ResMut<EventHandle>, mut events: EventReader<SendEvent>) {
    for ev in events.iter() {
        if let Err(e) = handle.sender.try_send(ev.0.clone()) {
            error!("Error sending event: {:?}", e);
        }
    }
}
