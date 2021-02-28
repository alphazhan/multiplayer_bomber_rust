use crate::utils;
use gdnative::api::*;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Control)]
pub struct Lobby {
    /// Panel
    connect: Option<Ref<Node>>,
    /// LineEdit
    connect_name: Option<Ref<Node>>,
    /// Label
    connect_error_label: Option<Ref<Node>>,
    /// LineEdit
    connect_address: Option<Ref<Node>>,
    /// Button
    connect_host: Option<Ref<Node>>,
    /// Button
    connect_join: Option<Ref<Node>>,

    /// AcceptDialog
    error_dialog: Option<Ref<Node>>,

    /// Panel
    players: Option<Ref<Node>>,

    /// ItemList
    players_list: Option<Ref<Node>>,

    /// Button
    players_start: Option<Ref<Node>>,
}

#[methods]
#[allow(deprecated)]
impl Lobby {
    fn new(_owner: TRef<Control>) -> Self {
        Lobby {
            connect: None,
            connect_name: None,
            connect_error_label: None,
            connect_address: None,
            connect_host: None,
            connect_join: None,

            error_dialog: None,

            players: None,
            players_list: None,
            players_start: None,
        }
    }

    #[export]
    fn _ready(&mut self, owner: TRef<Control>) -> () {
        self.update_child_nodes(owner);
        if let Err(e) = self.connect_signals(owner) {
            godot_error!("`Lobby` => GodotError at `connect_signals` function: {}", e);
        }
    }

    fn update_child_nodes(&mut self, owner: TRef<Control>) -> () {
        self.connect = owner.get_node("Connect");
        self.connect_name = owner.get_node("Connect/Name");
        self.connect_error_label = owner.get_node("Connect/ErrorLabel");
        self.connect_address = owner.get_node("Connect/IPAddress");
        self.connect_host = owner.get_node("Connect/Host");
        self.connect_join = owner.get_node("Connect/Join");
        self.error_dialog = owner.get_node("ErrorDialog");
        self.players = owner.get_node("Players");
        self.players_list = owner.get_node("Players/List");
        self.players_start = owner.get_node("Players/Start");
    }

    fn connect_signals(&self, owner: TRef<Control>) -> Result<(), GodotError> {
        let players_start = self.get_players_start();
        let connect_host = self.get_connect_host();
        let connect_join = self.get_connect_join();

        players_start.connect(
            "pressed",
            owner,
            "_on_start_pressed",
            VariantArray::new_shared(),
            0,
        )?;

        connect_host.connect(
            "pressed",
            owner,
            "_on_host_pressed",
            VariantArray::new_shared(),
            0,
        )?;

        connect_join.connect(
            "pressed",
            owner,
            "_on_join_pressed",
            VariantArray::new_shared(),
            0,
        )?;

        Ok(())
    }

    #[export]
    fn _on_host_pressed(&self, owner: TRef<Control>) -> () {
        let connect = self.get_connect();
        let connect_name = self.get_connect_name();
        let connect_error_label = self.get_connect_error_label();
        let players = self.get_players();

        if connect_name.text().to_string() == "" {
            connect_error_label.set_text("Invalid name!");
            return;
        }

        connect.hide();
        players.show();
        connect_error_label.set_text("");

        let gamestate = unsafe { utils::get_gamestate_singleton(owner.as_ref()) };
        let func_args = VariantArray::new_shared();
        unsafe {
            func_args.push(connect_name.text());
            gamestate.callv("host_game", func_args);
        }

        self.refresh_lobby(owner);
    }

    #[export]
    fn _on_join_pressed(&self, owner: TRef<Control>) -> () {
        let gamestate = unsafe { utils::get_gamestate_singleton(owner.as_ref()) };

        let connect_name = self.get_connect_name();
        let connect_error_label = self.get_connect_error_label();
        let connect_address = self.get_connect_address();
        let connect_host = self.get_connect_host();
        let connect_join = self.get_connect_join();

        if connect_name.text().to_string() == "" {
            connect_error_label.set_text("Invalid name!");
            return;
        }

        let ip = connect_address.text();
        if !ip.is_valid_ip_address() {
            connect_error_label.set_text("Invalid IP address!");
            return;
        }

        connect_error_label.set_text("");
        connect_host.set_disabled(true);
        connect_join.set_disabled(true);

        let player_name = connect_name.text();
        let func_args = VariantArray::new_shared();
        unsafe {
            func_args.push(ip);
            func_args.push(player_name);
            gamestate.callv("join_game", func_args);
        }
    }

    fn _on_connection_success(&self, owner: TRef<Control>) -> () {
        let connect = self.get_connect();

        let players = self.get_players();

        connect.hide();
        players.show();

        owner.emit_signal("connection_succeeded", &[]);
    }

    fn _on_connection_failed(&self, _owner: TRef<Control>) -> () {
        let connect_host = self.get_connect_host();
        let connect_join = self.get_connect_join();
        let connect_error_label = self.get_connect_error_label();

        connect_host.set_disabled(false);
        connect_join.set_disabled(false);
        connect_error_label.set_text("Connection failed.");
    }

    #[export]
    fn game_ended(&self, owner: TRef<Control>) -> () {
        let connect = self.get_connect();
        let players = self.get_players();
        let players_list = self.get_players_list();
        let connect_host = self.get_connect_host();
        let connect_join = self.get_connect_join();
        let world = unsafe { utils::get_world(owner.as_ref()) };

        world.queue_free();
        owner.show();
        connect.show();
        players.hide();
        players_list.clear();
        connect_host.set_disabled(false);
        connect_join.set_disabled(false);
    }

    #[export]
    fn game_error(&self, owner: TRef<Control>, error: String) -> () {
        godot_print!("Game error: {}", error);

        self.game_ended(owner);

        let error_dialog = self.get_error_dialog();
        let connect_host = self.get_connect_host();
        let connect_join = self.get_connect_join();

        error_dialog.set_text(error);
        error_dialog.popup_centered_minsize(Vector2::new(0.0, 0.0));
        connect_host.set_disabled(false);
        connect_join.set_disabled(false);
    }

    #[export]
    fn refresh_lobby(&self, owner: TRef<Control>) -> () {
        godot_print!("Refreshing lobby...");

        let tree = unsafe { utils::get_tree(owner.as_ref()) };
        let gamestate = unsafe { utils::get_gamestate_singleton(owner.as_ref()) };
        let gamestate_players = gamestate.get("players").to_dictionary();

        godot_print!("gamestate_players: {:?}", gamestate_players);

        let players_list = self.get_players_list();

        let players_start = self.get_players_start();

        players_list.clear();

        for (p_id, p_name) in gamestate_players.iter() {
            godot_print!("p: {}", p_name.to_string());
            players_list.add_item(
                p_name.try_to_string().unwrap_or("???".to_string())
                    + (if p_id.to_i64() == tree.get_network_unique_id() {
                        " (You)"
                    } else {
                        ""
                    }),
                Null::null(),
                true,
            );
        }

        players_start.set_disabled(!tree.is_network_server());

        godot_print!("Lobby was refreshed!");
    }

    #[export]
    fn change_to_players_lobby(&self, _owner: TRef<Control>) -> () {
        let connect = self.get_connect();
        let players = self.get_players();

        connect.hide();
        players.show()
    }

    #[export]
    fn _on_start_pressed(&self, owner: TRef<Control>) -> () {
        unsafe {
            utils::get_gamestate_singleton(owner.as_ref())
                .callv("start_game", VariantArray::new_shared());
        }
    }

    #[export]
    fn _on_find_public_ip_pressed(&self, _owner: TRef<Control>) -> () {
        godot_print!("https://icanhazip.com/");
    }

    // get child nodes

    fn get_connect(&self) -> TRef<Panel> {
        let connect = self.connect.unwrap();
        let connect = unsafe { connect.assume_safe() };
        let connect = connect.cast::<Panel>().unwrap();

        return connect;
    }

    fn get_connect_name(&self) -> TRef<LineEdit> {
        let connect_name = self.connect_name.unwrap();
        let connect_name = unsafe { connect_name.assume_safe() };
        let connect_name = connect_name.cast::<LineEdit>().unwrap();

        return connect_name;
    }

    fn get_connect_error_label(&self) -> TRef<Label> {
        let connect_error_label = self.connect_error_label.unwrap();
        let connect_error_label = unsafe { connect_error_label.assume_safe() };
        let connect_error_label = connect_error_label.cast::<Label>().unwrap();

        return connect_error_label;
    }

    fn get_connect_address(&self) -> TRef<LineEdit> {
        let connect_address = self.connect_address.unwrap();
        let connect_address = unsafe { connect_address.assume_safe() };
        let connect_address = connect_address.cast::<LineEdit>().unwrap();

        return connect_address;
    }

    fn get_connect_host(&self) -> TRef<Button> {
        let connect_host = self.connect_host.unwrap();
        let connect_host = unsafe { connect_host.assume_safe() };
        let connect_host = connect_host.cast::<Button>().unwrap();

        return connect_host;
    }

    fn get_connect_join(&self) -> TRef<Button> {
        let connect_join = self.connect_join.unwrap();
        let connect_join = unsafe { connect_join.assume_safe() };
        let connect_join = connect_join.cast::<Button>().unwrap();

        return connect_join;
    }

    fn get_error_dialog(&self) -> TRef<AcceptDialog> {
        let error_dialog = self.error_dialog.unwrap();
        let error_dialog = unsafe { error_dialog.assume_safe() };
        let error_dialog = error_dialog.cast::<AcceptDialog>().unwrap();

        return error_dialog;
    }

    fn get_players(&self) -> TRef<Panel> {
        let players = self.players.unwrap();
        let players = unsafe { players.assume_safe() };
        let players = players.cast::<Panel>().unwrap();

        return players;
    }

    fn get_players_list(&self) -> TRef<ItemList> {
        let players_list = self.players_list.unwrap();
        let players_list = unsafe { players_list.assume_safe() };
        let players_list = players_list.cast::<ItemList>().unwrap();

        return players_list;
    }

    fn get_players_start(&self) -> TRef<Button> {
        let players_start = self.players_start.unwrap();
        let players_start = unsafe { players_start.assume_safe() };
        let players_start = players_start.cast::<Button>().unwrap();

        return players_start;
    }
}
