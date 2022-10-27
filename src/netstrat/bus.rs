use std::collections::HashMap;

use crossbeam::channel::{Receiver, Sender};
use std::time::Duration;
use tracing::{debug, error};

use super::{errors, Message};

#[derive(Default, Clone)]
pub struct Bus {
    channels: HashMap<String, (Sender<Message>, Receiver<Message>)>,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            channels: Default::default(),
        }
    }

    pub fn read(&self, ch_name: String) -> Result<Message, errors::Bus> {
        debug!("reading from channel: {ch_name}");

        let receiver = self.channel_get(ch_name)?.1;
        let msg = receiver.recv_timeout(Duration::from_nanos(1))?;

        debug!("successfully read from channel: {msg:?}");

        Ok(msg)
    }

    pub fn write(&mut self, ch_name: String, msg: Message) -> Result<(), errors::Bus> {
        debug!("writing to channel; channel: {ch_name}, message: {msg:?}");
        let sender = self.channel_get(ch_name)?.0;
        debug!("successfully sent to channel");
        Ok(sender.send(msg)?)
    }

    fn channel_get(
        &self,
        ch_name: String,
    ) -> Result<(Sender<Message>, Receiver<Message>), errors::Bus> {
        let channel_wrapped = self.channels.get(&ch_name);
        if channel_wrapped.is_none() {
            error!("channel not found: {ch_name}");
            return Err(errors::Bus::ChannelNotFound(ch_name));
        };

        Ok(channel_wrapped.unwrap().clone())
    }
}
