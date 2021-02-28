use gdnative::prelude::*;

mod gamestate;
mod lobby;
mod score;

mod bomb;
mod player;
mod rock;

mod utils;

fn init(handle: InitHandle) -> () {
    handle.add_class::<gamestate::GameState>();
    handle.add_class::<lobby::Lobby>();
    handle.add_class::<score::Score>();
    handle.add_class::<player::Player>();
    handle.add_class::<bomb::Bomb>();
    handle.add_class::<rock::Rock>();
}

godot_init!(init);
