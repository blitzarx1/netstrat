use std::{collections::HashMap, rc::Rc, sync::Mutex};

use crossbeam::channel::{unbounded, Receiver, Sender};
use std::time::Duration;
use tracing::{debug, error, trace};

use super::{errors, Message};

#[derive(Default, Clone)]
pub struct Bus {
    channels: Rc<Mutex<HashMap<String, (Sender<Message>, Receiver<Message>)>>>,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            channels: Default::default(),
        }
    }

    pub fn read(&mut self, ch_name: String) -> Result<Message, errors::Bus> {
        trace!("reading from channel: {ch_name}");

        let receiver = self.channel_get_or_create(ch_name).1;
        let msg = receiver.recv_timeout(Duration::from_nanos(1))?;

        trace!("successfully read from channel: {msg:?}");

        Ok(msg)
    }

    pub fn write(&mut self, ch_name: String, msg: Message) -> Result<(), errors::Bus> {
        trace!("writing to channel; channel: {ch_name}, message: {msg:?}");

        let sender = self.channel_get_or_create(ch_name).0;
        sender.send(msg)?;

        trace!("successfully sent to channel");

        Ok(())
    }

    fn channel_get_or_create(&mut self, ch_name: String) -> (Sender<Message>, Receiver<Message>) {
        let mut locked_channesls = self.channels.lock().unwrap();
        let channel_wrapped = locked_channesls.get(&ch_name);
        if channel_wrapped.is_none() {
            debug!("channel not found: {ch_name}; creating new...");

            let res = unbounded();
            locked_channesls.insert(ch_name, res.clone());
            return res;
        };

        channel_wrapped.unwrap().clone()
    }
}
