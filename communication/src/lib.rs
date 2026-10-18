use robolab_bot_base::communication::BotCommunication;
use rumqttc::Transport;
use std::time::Duration;

#[cfg(test)]
mod tests;

mod communication_impl;

pub use communication_impl::{Communication, From, Payload, Typ, *};

// WARN: DO NOT EDIT FUNCTION SIGNATURE.
// You can edit the body of this function however you want but the function signature MUST NOT be changed.
//
// This function and your Message type is the only public interface of this library.
pub fn create_communication() -> Box<dyn BotCommunication<M = Message>> {
    let mut opts = rumqttc::MqttOptions::new("004", "mothership.inf.tu-dresden.de", 8883);
    opts.set_transport(Transport::tls_with_default_config())
        .set_clean_session(true)
        .set_keep_alive(Duration::from_secs(120))
        .set_credentials("004", "<redacted>");
    let (client, connection) = rumqttc::Client::new(opts, 1 << 12);
    let comm = Communication::new(client, connection);
    Box::new(comm)
}

// WARN: DO NOT EDIT
// You can edit the message type however you want but this export and the type name MUST NOT be changed.
//
// The above function and your Message type is the only public interface of this library.
pub use communication_impl::Message;
