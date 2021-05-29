use gdnative::api::*;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Area2D)]
pub struct Bomb {
    // preloads
    #[property]
    in_area: VariantArray,
    #[property]
    from_player_id: i64,
}

#[methods]
#[allow(deprecated)]
impl Bomb {
    fn new(_owner: TRef<Area2D>) -> Self {
        Bomb {
            in_area: VariantArray::new_shared(),
            from_player_id: 0,
        }
    }

    #[export]
    fn _ready(&mut self, owner: TRef<Area2D>) {
        if let Err(e) = self.connect_signals(owner) {
            godot_error!("`Bomb` => GodotError at `connect_signals` function: {}", e);
        }
    }

    #[export]
    fn _on_bomb_body_entered(&self, _owner: TRef<Area2D>, body: Variant) {
        // Body
        let body = body.try_to_object::<Node>().unwrap();
        let body = unsafe { body.assume_safe() };
        //

        if !self.in_area.contains(body) {
            unsafe {
                self.in_area.push(body);
            }
        }
    }

    #[export]
    fn _on_bomb_body_exited(&self, _owner: TRef<Area2D>, body: Variant) {
        // Body
        let body = body.try_to_object::<Node>().unwrap();
        let body = unsafe { body.assume_safe() };
        //

        unsafe {
            self.in_area.erase(body);
        }
    }

    #[export(rpc = "master")]
    fn explode(&self, owner: TRef<Area2D>) {
        if !owner.is_network_master() {
            godot_warn!("`explode` function is only available for `master`!");
            return;
        }

        for object in self.in_area.iter() {
            if object.has_method("exploded") {
                // Exploded has a master keyword, so it will only be received by the master.
                // Player
                let object = object.try_to_object::<Node>().unwrap();
                let object = unsafe { object.assume_safe() };
                //

                println!(
                    "self.from_player_id: {:?}",
                    self.from_player_id.to_variant()
                );
                object.rpc("exploded", &[self.from_player_id.to_variant()]);
            }
        }
    }

    fn connect_signals(&self, owner: TRef<Area2D>) -> Result<(), GodotError> {
        owner.connect(
            "body_entered",
            owner,
            "_on_bomb_body_entered",
            VariantArray::new_shared(),
            0,
        )?;

        owner.connect(
            "body_exited",
            owner,
            "_on_bomb_body_exited",
            VariantArray::new_shared(),
            0,
        )?;

        Ok(())
    }

    #[export]
    fn done(&self, owner: TRef<Area2D>) {
        owner.queue_free();
    }
}
