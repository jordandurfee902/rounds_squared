use bevy::prelude::*;
use matchbox_socket::WebRtcSocket;

pub mod rng;
pub mod protocol;
pub mod matchmaking;
pub mod host;
pub mod client;

pub use rng::*;
pub use protocol::*;
pub use matchmaking::*;
pub use host::*;
pub use client::*;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalPlayerIndex(pub usize);

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct IsNetworked(pub bool);

#[derive(Resource, Deref, DerefMut)]
pub struct MatchboxSocketResource(pub WebRtcSocket);

#[derive(Resource, Default, Debug, Clone)]
pub struct OnlineCodeResource {
    pub code: String, // Empty String means public matchmaking
    pub is_host: bool,
}
