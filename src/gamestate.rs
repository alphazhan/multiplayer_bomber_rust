use rand::{self, Rng};

use crate::utils;
use gdnative::api::*;
use gdnative::prelude::*;

/// Default game server port. Can be any number between 1024 and 49151.
/// Not on the list of registered or common ports as of November 2020:
/// https://en.wikipedia.org/wiki/List_of_TCP_and_UDP_port_numbers
const DEFAULT_PORT: i64 = 10567;

/// Max number of players.
const MAX_PEERS: i64 = 12;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct GameState {
    // PackedScenes
    #[property]
    preload_world: Ref<PackedScene>,
    #[property]
    preload_player: Ref<PackedScene>,

    /// Name for my player.
    #[property]
    player_name: String,

    /// Names for remote players in id:name format.
    #[property]
    players: Dictionary,
}

#[methods]
#[allow(deprecated)]
impl GameState {
    fn new(_owner: TRef<Node>) -> Self {
        GameState {
            preload_world: PackedScene::new().into_shared(),
            preload_player: PackedScene::new().into_shared(),

            player_name: "The Warrior".to_string(),
            players: Dictionary::new().into_shared(),
        }
    }

    #[export]
    fn _ready(&mut self, owner: TRef<Node>) -> () {
        self.preload_instances();

        if let Err(e) = self.connect_network_signals(owner) {
            godot_error!(
                "`Gamestate` => GodotError at `connect_network_signals` function: {}",
                e
            );
        }
    }

    #[export]
    fn host_game(&mut self, owner: TRef<Node>, player_name: Variant) -> () {
        godot_print!("Hosting game...");

        let tree = unsafe { utils::get_tree(owner.as_ref()) };

        let host = NetworkedMultiplayerENet::new();
        if let Err(e) = host.create_server(DEFAULT_PORT, MAX_PEERS, 0, 0) {
            godot_error!(
                "`Gamestate` => GodotError at `create_server` function: {}",
                e
            );
        }

        self.player_name = player_name.to_string();

        tree.set_network_peer(host);

        self.self_register_player(owner);

        godot_print!("Game hosted!");
    }

    #[export]
    fn join_game(&mut self, owner: TRef<Node>, ip: Variant, player_name: Variant) -> () {
        godot_print!("Joining to the game");

        let tree = unsafe { utils::get_tree(owner.as_ref()) };

        let client = NetworkedMultiplayerENet::new();
        if let Err(e) = client.create_client(ip.to_godot_string(), DEFAULT_PORT, 0, 0, 0) {
            godot_error!(
                "`Gamestate` => GodotError at `create_client` function: {} ({})",
                e,
                player_name.to_string()
            );
        }

        self.player_name = player_name.to_string();

        tree.set_network_peer(client);

        godot_print!("Joined the game successfully!");
    }

    fn connect_network_signals(&self, owner: TRef<Node>) -> Result<(), GodotError> {
        let tree = unsafe { utils::get_tree(owner.as_ref()) };

        // Connecting network signals
        tree.connect(
            "network_peer_connected",
            owner,
            "_player_connected",
            VariantArray::new_shared(),
            0,
        )?;
        tree.connect(
            "network_peer_disconnected",
            owner,
            "_player_disconnected",
            VariantArray::new_shared(),
            0,
        )?;
        tree.connect(
            "connected_to_server",
            owner,
            "_connected_ok",
            VariantArray::new_shared(),
            0,
        )?;
        tree.connect(
            "connection_failed",
            owner,
            "_connected_fail",
            VariantArray::new_shared(),
            0,
        )?;
        tree.connect(
            "server_disconnected",
            owner,
            "_server_disconnected",
            VariantArray::new_shared(),
            0,
        )?;

        Ok(())
    }

    /// # The First Step
    /// Rpc-ing `create_world` to all clients and to server
    #[export]
    fn start_game(&mut self, owner: TRef<Node>) -> () {
        // Tree
        let tree = unsafe { utils::get_tree(owner.as_ref()) };

        if tree.is_network_server() {
            self.create_world(owner);
            owner.rpc("create_world", &[]);
        }
    }

    /// # Second step
    /// Creating world
    #[export(rpc = "remote")]
    fn create_world(&self, owner: TRef<Node>) -> () {
        godot_print!("creating world...");

        // let tree = unsafe { utils::get_tree(owner.as_ref()) };
        let root = unsafe { utils::get_root(owner.as_ref()) };

        // World
        let world_packed_scene = unsafe { self.preload_world.assume_safe() };
        let world_packed_scene = world_packed_scene.cast::<PackedScene>().unwrap();

        // instancing world packed scene
        let world = world_packed_scene
            .instance(PackedScene::GEN_EDIT_STATE_DISABLED)
            .unwrap();
        let world = unsafe { world.assume_safe() };
        //

        root.add_child(world, false);
        //

        // Player
        let player_packed_scene = unsafe { self.preload_player.assume_safe() };
        let player_packed_scene = player_packed_scene.cast::<PackedScene>().unwrap();
        //

        // Score
        let score = world.get_node("Score").unwrap();
        let score = unsafe { score.assume_safe() };
        let score = score.cast::<HBoxContainer>().unwrap();
        //

        // Players
        let players = world.get_node("Players").unwrap();
        let players = unsafe { players.assume_safe() };
        let players = players.cast::<Node2D>().unwrap();
        //

        let mut rng = rand::thread_rng();

        for (player_id, player_name) in self.players.iter() {
            godot_print!("creating {} player...", player_id.to_i64());

            // instancing player packed scene
            let new_player = player_packed_scene
                .instance(PackedScene::GEN_EDIT_STATE_DISABLED)
                .unwrap();
            let new_player = unsafe { new_player.assume_safe() };
            let new_player = new_player.cast::<KinematicBody2D>().unwrap();
            //

            new_player.set_name(player_id.to_godot_string()); // Use unique ID as node name.
            new_player.set_network_master(player_id.to_i64(), true);

            unsafe {
                if player_id.to_i64() == utils::get_tree(owner.as_ref()).get_network_unique_id() {
                    // world_spawn_point
                    let world_spawn_point: TRef<Position2D> = world
                        .get_node(format!("SpawnPoints/{}", rng.gen_range(0..=11)))
                        .unwrap()
                        .assume_safe()
                        .cast::<Position2D>()
                        .unwrap();
                    //

                    new_player.set_position(world_spawn_point.position());
                }
            }

            let func_args = VariantArray::new_shared();
            unsafe {
                func_args.push(player_name.clone());
            }
            new_player.callv("set_player_name", func_args);

            players.add_child(new_player, false);

            unsafe {
                self.players.insert(player_id.to_i64(), new_player);
            }

            let func_args = VariantArray::new_shared();
            unsafe {
                func_args.push(player_id.clone());
                func_args.push(player_name.clone());
            }

            score.callv("add_player", func_args);

            godot_print!("player {} created!", player_id.to_i64());
        }

        let lobby = unsafe { utils::get_lobby(owner.as_ref()) };
        lobby.hide();

        godot_print!("world created!");
    }

    // Network signals

    /// Callback from SceneTree.
    #[export]
    fn _player_connected(&self, owner: TRef<Node>, id: i64) -> () {
        godot_print!("New player (id: {}) connected", id);

        // Tree
        let tree = unsafe { utils::get_tree(owner.as_ref()) };

        // Registration of a client beings here,
        // tell the connected player that we are here.
        owner.rpc_id(
            id,
            "register_player",
            &[
                tree.get_network_unique_id().to_variant(),
                self.player_name.to_variant(),
            ],
        );
    }

    /// Callback from SceneTree.
    /// player disconnected
    #[export]
    fn _player_disconnected(&self, owner: TRef<Node>, id: i64) -> () {
        godot_print!("Player (id: {}) disconnected", id);

        unsafe {
            if utils::get_lobby(owner.as_ref()).is_visible() == false {
                self.game_error(owner, "Player disconnected");
            }
        }

        self.unregister_player(owner, id);
    }

    /// Callback from SceneTree, only for clients (not server).
    /// We just connected to a server
    #[export]
    fn _connected_ok(&self, owner: TRef<Node>) -> () {
        godot_print!("user connected to the server successfully");

        self.self_register_player(owner);
    }

    /// Callback from SceneTree, only for clients (not server).
    #[export]
    fn _server_disconnected(&self, owner: TRef<Node>) -> () {
        self.game_error(owner, "Server disconnected");
    }

    /// Callback from SceneTree, only for clients (not server).
    #[export]
    fn _connected_fail(&self, owner: TRef<Node>) -> () {
        self.game_error(owner, "User connected to the server failure");
    }

    fn game_error(&self, owner: TRef<Node>, error: &str) -> () {
        let tree = unsafe { utils::get_tree(owner.as_ref()) };
        tree.set_network_peer(Null::null()); // Remove peer

        unsafe {
            self.players.clear();
        }

        let lobby = unsafe { utils::get_lobby(owner.as_ref()) };
        let func_args = VariantArray::new_shared();
        unsafe {
            func_args.push(error);
        }

        lobby.callv("game_error", func_args);
    }

    #[export]
    fn end_game(&self, owner: TRef<Node>) -> () {
        let tree = unsafe { utils::get_tree(owner.as_ref()) };
        tree.set_network_peer(Null::null()); // Remove peer

        unsafe {
            self.players.clear();
        }

        unsafe {
            utils::get_world(owner.as_ref()).queue_free();
            utils::get_lobby(owner.as_ref()).callv("game_ended", VariantArray::new_shared());
        }
    }

    // Register the new player
    #[export(rpc = "remote")]
    fn register_player(&self, owner: TRef<Node>, id: Variant, p_name: Variant) -> () {
        godot_print!(
            "register player {} (id:{})",
            p_name.to_string(),
            id.to_i64()
        );

        unsafe {
            self.players.insert(id, p_name);
        }

        let lobby = unsafe { utils::get_lobby(owner.as_ref()) };
        lobby.callv("refresh_lobby", VariantArray::new_shared());
    }

    fn unregister_player(&self, owner: TRef<Node>, id: i64) -> () {
        godot_print!("unregister player id:{}", id);

        unsafe {
            self.players.erase(id);
        }

        let lobby = unsafe { utils::get_lobby(owner.as_ref()) };
        lobby.callv("refresh_lobby", VariantArray::new_shared());
    }

    fn self_register_player(&self, owner: TRef<Node>) -> () {
        let lobby = unsafe { utils::get_lobby(owner.as_ref()) };
        lobby.callv("change_to_players_lobby", VariantArray::new_shared());

        let tree = unsafe { utils::get_tree(owner.as_ref()) };
        self.register_player(
            owner,
            tree.get_network_unique_id().to_variant(),
            self.player_name.to_variant(),
        );
    }

    /// Preloading `PackedScene` instances (World, Player)
    fn preload_instances(&mut self) -> () {
        // World
        let world_scene = ResourceLoader::godot_singleton()
            .load("res://scenes/World/World.tscn", "PackedScene", false)
            .unwrap();
        let world_scene = unsafe { world_scene.assume_unique().into_shared() };

        self.preload_world = world_scene.cast::<PackedScene>().unwrap();

        // Player
        let player_scene = ResourceLoader::godot_singleton()
            .load("res://scenes/Player/Player.tscn", "PackedScene", false)
            .unwrap();
        let player_scene = unsafe { player_scene.assume_unique().into_shared() };

        self.preload_player = player_scene.cast::<PackedScene>().unwrap();
    }
}
