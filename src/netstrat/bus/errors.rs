use crossbeam::channel::{RecvTimeoutError, SendError};
use quick_error::quick_error;

use super::Message;

quick_error! {
    #[derive(Debug)]
    pub enum Bus {
        ChannelNotFound(name: String) {
            display("channel not found: {name}")
        }
        Recv(err: RecvTimeoutError) {
            from()
            display("{}", err)
        }
        Send(err: SendError<Message>) {
            from()
            display("{}", err)
        }
    }
}
