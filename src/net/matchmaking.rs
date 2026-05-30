use bevy::prelude::*;
use matchbox_socket::WebRtcSocket;
use crate::settings::GameState;
use crate::maps::ActiveMap;
use super::{MatchboxSocketResource, OnlineCodeResource, LocalPlayerIndex, IsNetworked, RollbackRng, HostPeerId, HostStatePacket};

pub fn start_matchmaking(
    mut commands: Commands,
    code_res: Option<Res<OnlineCodeResource>>,
) {
    let room_url = if let Some(res) = code_res {
        if res.code.is_empty() {
            info!("MATCHMAKING: Connecting to public matchmaking lobby...");
            "wss://durfdog-sets.hf.space/rounds_lobby?next=8".to_string()
        } else {
            let formatted_code = res.code.replace(" ", "");
            info!("MATCHMAKING: Connecting to private room code: {} (is_host: {})", formatted_code, res.is_host);
            format!("wss://durfdog-sets.hf.space/room_{}?next=8", formatted_code)
        }
    } else {
        info!("MATCHMAKING: Connecting to public matchmaking lobby (fallback)...");
        "wss://durfdog-sets.hf.space/rounds_lobby?next=8".to_string()
    };
    
    let (socket, message_loop) = WebRtcSocket::builder(&room_url)
        .add_channel(matchbox_socket::ChannelConfig::unreliable())
        .build();
    
    bevy::tasks::IoTaskPool::get().spawn(message_loop).detach();
    commands.insert_resource(MatchboxSocketResource(socket));
}

pub fn lobby_system(
    mut commands: Commands,
    socket_res: Option<ResMut<MatchboxSocketResource>>,
    mut next_state: ResMut<NextState<GameState>>,
    code_res: Option<Res<OnlineCodeResource>>,
) {
    let Some(mut socket) = socket_res else {
        return;
    };

    socket.update_peers();

    let is_private = code_res.map(|r| !r.code.is_empty()).unwrap_or(false);

    if is_private {
        // Private Match Client: Wait for a HostStatePacket from the host to identify them.
        let packets = socket.channel_mut(0).receive();
        for (peer, data) in packets {
            if let Ok(_pkt) = serde_json::from_slice::<HostStatePacket>(&data) {
                info!("MATCHMAKING: Found private match host: {:?}", peer);
                
                let local_id = socket.id().expect("Socket should have an ID");
                
                // Get all clients (excluding host) sorted alphabetically to determine local index
                let mut client_ids = Vec::new();
                if local_id != peer {
                    client_ids.push(local_id);
                }
                for other_peer in socket.connected_peers() {
                    if other_peer != peer {
                        client_ids.push(other_peer);
                    }
                }
                client_ids.sort_by_key(|id| id.to_string());
                
                let mut local_player_idx = 1;
                for (i, id) in client_ids.iter().enumerate() {
                    if *id == local_id {
                        local_player_idx = i + 1;
                    }
                }
                
                commands.insert_resource(HostPeerId(peer));
                commands.insert_resource(LocalPlayerIndex(local_player_idx));
                commands.insert_resource(IsNetworked(true));
                commands.insert_resource(RollbackRng::new(98765));
                
                info!("MATCHMAKING: Client joined private lobby. Host: {:?}, Local Index: {}", peer, local_player_idx);
                next_state.set(GameState::Lobby);
                return;
            }
        }
    } else {
        // Public Matchmaking: Wait for at least one peer, sort alphabetically
        let connected_peers = socket.connected_peers().count();
        if connected_peers < 1 {
            return;
        }

        info!("MATCHMAKING: Peer connected! Establishing Host-Client relationship...");

        let local_id = socket.id().expect("Socket should have an ID");
        let mut all_ids = vec![local_id];
        for peer in socket.connected_peers() {
            all_ids.push(peer);
        }
        all_ids.sort_by_key(|id| id.to_string());

        let mut local_player_idx = 0;
        for (i, id) in all_ids.iter().enumerate() {
            if *id == local_id {
                local_player_idx = i;
            }
        }

        if local_player_idx > 0 {
            // Client: first peer (all_ids[0]) is the host
            commands.insert_resource(HostPeerId(all_ids[0]));
        }

        commands.insert_resource(LocalPlayerIndex(local_player_idx));
        commands.insert_resource(IsNetworked(true));
        commands.insert_resource(RollbackRng::new(98765));

        info!("MATCHMAKING: Connection verified! Local Player Index: {}", local_player_idx);
        next_state.set(GameState::Lobby);
    }
}

pub fn parse_game_state(name: &str) -> GameState {
    match name {
        "MainMenu" => GameState::MainMenu,
        "Lobby" => GameState::Lobby,
        "OnlineMenu" => GameState::OnlineMenu,
        "Matchmaking" => GameState::Matchmaking,
        "Gameplay" => GameState::Gameplay,
        "CardSelection" => GameState::CardSelection,
        _ => GameState::Gameplay,
    }
}

pub fn parse_active_map(name: &str) -> ActiveMap {
    match name {
        "DefaultMap" => ActiveMap::DefaultMap,
        "PillarsMap" => ActiveMap::PillarsMap,
        "StadiumMap" => ActiveMap::StadiumMap,
        "ChasmBridge" => ActiveMap::ChasmBridge,
        "Gridlock" => ActiveMap::Gridlock,
        "Hourglass" => ActiveMap::Hourglass,
        "IceTemple" => ActiveMap::IceTemple,
        "IndustrialFoundry" => ActiveMap::IndustrialFoundry,
        "VerticalHelix" => ActiveMap::VerticalHelix,
        "TectonicFissure" => ActiveMap::TectonicFissure,
        "ZenGarden" => ActiveMap::ZenGarden,
        "SpaceStation" => ActiveMap::SpaceStation,
        "AncientColiseum" => ActiveMap::AncientColiseum,
        _ => ActiveMap::DefaultMap,
    }
}
