use std::f64::consts;

use crate::utils;
use gdnative::api::*;
use gdnative::prelude::*;

const MOTION_SPEED: f32 = 90.0;

#[derive(NativeClass)]
#[inherit(KinematicBody2D)]
pub struct Player {
    // preloads
    #[property]
    preload_bomb: Ref<PackedScene>,

    /// Current animation
    current_anim: String,
    prev_bombing: bool,
    bomb_index: u16,
    #[property]
    stunned: bool,
}

#[methods]
impl Player {
    fn new(_owner: TRef<KinematicBody2D>) -> Self {
        Player {
            preload_bomb: PackedScene::new().into_shared(),
            current_anim: "".to_string(),
            prev_bombing: false,
            bomb_index: 0,
            stunned: false,
        }
    }

    #[export]
    fn _ready(&mut self, _owner: TRef<KinematicBody2D>) {
        self.preload_instances();
    }

    #[export]
    fn _physics_process(&mut self, owner: TRef<KinematicBody2D>, _delta: f64) {
        if owner.is_network_master() {
            let mut motion = Vector2::zero();

            let input = Input::godot_singleton();
            if Input::is_action_pressed(input, "ui_left") {
                motion.x -= 1.0;
            }
            if Input::is_action_pressed(input, "ui_right") {
                motion.x += 1.0;
            }
            if Input::is_action_pressed(input, "ui_up") {
                motion.y -= 1.0;
            }
            if Input::is_action_pressed(input, "ui_down") {
                motion.y += 1.0;
            }

            let mut bombing = Input::is_action_pressed(input, "ui_select");

            if self.stunned {
                bombing = false;
                motion = Vector2::zero();
            }

            if bombing && !self.prev_bombing {
                let bomb_name = format!("{}{}", owner.name(), self.bomb_index);
                let bomb_pos = owner.position();
                let network_unique_id =
                    unsafe { utils::get_tree(owner.as_ref()).get_network_unique_id() };

                self.setup_bomb(
                    owner,
                    bomb_name.to_variant(),
                    bomb_pos.to_variant(),
                    network_unique_id.to_variant(),
                );
                owner.rpc(
                    "setup_bomb",
                    &[
                        bomb_name.to_variant(),
                        bomb_pos.to_variant(),
                        network_unique_id.to_variant(),
                    ],
                );
            }

            self.prev_bombing = bombing;

            let mut new_anim = String::from("standing");
            if motion.y < 0.0 {
                new_anim = String::from("walk_up");
            } else if motion.y > 0.0 {
                new_anim = String::from("walk_down");
            } else if motion.x < 0.0 {
                new_anim = String::from("walk_left");
            } else if motion.x > 0.0 {
                new_anim = String::from("walk_right");
            }

            if self.stunned {
                new_anim = String::from("stunned");
            }

            if new_anim != self.current_anim {
                self.current_anim = new_anim;

                unsafe {
                    self.get_animation(owner)
                        .play(self.current_anim.clone(), -1.0, 1.0, false);
                }
            }

            owner.move_and_slide(
                motion * MOTION_SPEED,
                Vector2::new(0.0, 1.0),
                false,
                4,
                consts::FRAC_PI_4,
                true,
            );
        }

        if owner.is_network_master() {
            owner.rpc(
                "update_network",
                &[
                    owner.position().to_variant(),
                    self.current_anim.to_variant(),
                ],
            );
        }
    }

    // Updating position of the player
    #[export(rpc = "puppet")]
    fn update_network(
        &self,
        owner: TRef<KinematicBody2D>,
        position: Variant,
        current_anim: Variant,
    ) {
        owner.set_position(position.to_vector2());
        unsafe {
            self.get_animation(owner)
                .play(current_anim.to_string(), -1.0, 1.0, false);
        }
    }

    /// Remote function (You need to call the function like a `remotesync` mode.
    ///
    /// Example:
    /// ```
    /// setup_bomb(...);
    /// rpc("setup_bomb", ...);
    /// ```
    ///
    /// Create bomb
    /// `bomb_name`: String
    /// `bomb_pos`: Vector2
    /// `network_unique_id`: i64
    #[export(rpc = "remote")]
    fn setup_bomb(
        &self,
        owner: TRef<KinematicBody2D>,
        bomb_name: Variant,
        bomb_pos: Variant,
        network_unique_id: Variant,
    ) {
        let bomb_packed_scene = unsafe { self.preload_bomb.assume_safe() };

        // Bomb
        let bomb = bomb_packed_scene.instance(0).unwrap();
        let bomb = unsafe { bomb.assume_safe() };
        let bomb = bomb.cast::<Area2D>().unwrap();
        //

        // Bomb properties
        bomb.set_name(bomb_name.to_godot_string()); // Ensure unique name for the bomb
        bomb.set_position(bomb_pos.to_vector2());
        bomb.set("from_player_id", network_unique_id);
        //

        // No need to set network master to bomb, by default will be owned by the server
        let world = unsafe { utils::get_world(owner.as_ref()) };
        world.add_child(bomb, false);
    }

    #[export(rpc = "puppet")]
    fn stun(&mut self, _owner: TRef<KinematicBody2D>) {
        self.stunned = true
    }

    #[export(rpc = "master")]
    fn exploded(&mut self, owner: TRef<KinematicBody2D>, _by_who: Variant) {
        if self.stunned {
            return;
        }

        owner.rpc("stun", &[]); // Stun puppets
        self.stun(owner); // Stun master - could use sync to do both at once
    }

    #[export]
    fn set_player_name(&self, owner: TRef<KinematicBody2D>, player_name: Variant) {
        // `nickname` Label
        let nickname = owner.get_node("nickname").unwrap();
        let nickname = unsafe { nickname.assume_safe() };
        let nickname = nickname.cast::<Label>().unwrap();
        //

        nickname.set_text(player_name.to_godot_string());
    }

    fn preload_instances(&mut self) {
        let bomb_scene = ResourceLoader::godot_singleton()
            .load("res://scenes/Bomb/Bomb.tscn", "PackedScene", false)
            .unwrap();
        let bomb_scene = unsafe { bomb_scene.assume_unique().into_shared() };

        self.preload_bomb = bomb_scene.cast::<PackedScene>().unwrap();
    }

    unsafe fn get_animation(&self, owner: TRef<KinematicBody2D>) -> TRef<AnimationPlayer> {
        owner
            .get_node("anim")
            .unwrap()
            .assume_safe()
            .cast::<AnimationPlayer>()
            .unwrap()
    }
}
