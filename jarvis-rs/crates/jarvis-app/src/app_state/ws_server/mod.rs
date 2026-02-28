//! Mobile relay bridge: connects outbound to the relay server and streams
//! PTY output to/from connected mobile clients.

mod broadcast;
mod crypto_bridge;
mod mobile_polling;
mod pairing;
pub(crate) mod protocol;
mod relay_client;
mod relay_polling;
mod relay_protocol;
mod startup;

pub(crate) use broadcast::MobileBroadcaster;
pub(crate) use relay_client::{ClientCommand, RelayEvent};
