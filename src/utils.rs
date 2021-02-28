use gdnative::api::*;
use gdnative::prelude::*;

/// Get tree
pub unsafe fn get_tree(owner: &Node) -> TRef<SceneTree> {
    owner
        .get_tree()
        .unwrap()
        .assume_safe()
        .cast::<SceneTree>()
        .unwrap()
}

/// Get root
pub unsafe fn get_root(owner: &Node) -> TRef<Viewport> {
    get_tree(owner).root().unwrap().assume_safe()
}

/// Get gamestate singleton
pub unsafe fn get_gamestate_singleton(owner: &Node) -> TRef<Node> {
    get_root(owner)
        .get_node("gamestate")
        .unwrap()
        .assume_safe()
        .cast::<Node>()
        .unwrap()
}

/// Get lobby
pub unsafe fn get_lobby(owner: &Node) -> TRef<Control> {
    get_root(owner)
        .get_node("Lobby")
        .unwrap()
        .assume_safe()
        .cast::<Control>()
        .unwrap()
}

/// Get world
pub unsafe fn get_world(owner: &Node) -> TRef<Node2D> {
    get_root(owner)
        .get_node("World")
        .unwrap()
        .assume_safe()
        .cast::<Node2D>()
        .unwrap()
}

// godot_error!("`` => GodotError at `` function: {}", e);
