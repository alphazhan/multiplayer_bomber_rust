use crate::utils;
use gdnative::api::*;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(KinematicBody2D)]
pub struct Rock;

#[methods]
#[allow(deprecated)]
impl Rock {
    fn new(_owner: TRef<KinematicBody2D>) -> Self {
        Rock
    }

    /// Sent to everyone else
    #[export(rpc = "puppet")]
    fn do_explosion(&self, owner: TRef<KinematicBody2D>) -> () {
        // anim_player
        let anim_player = owner.get_node("AnimationPlayer").unwrap();
        let anim_player = unsafe { anim_player.assume_safe() };
        let anim_player = anim_player.cast::<AnimationPlayer>().unwrap();
        //

        anim_player.play("explode", -1.0, 1.0, false);
    }

    /// Received by owner of the rock
    #[export(rpc = "master")]
    fn exploded(&self, owner: TRef<KinematicBody2D>, by_who: Variant) -> () {
        owner.rpc("do_explosion", &[]); // Re-sent to puppet rocks

        let world = unsafe { utils::get_world(owner.as_ref()) };

        // Score
        let score = world.get_node("Score").unwrap();
        let score = unsafe { score.assume_safe() };
        let score = score.cast::<HBoxContainer>().unwrap();
        //

        let func_args = VariantArray::new_shared();
        unsafe {
            func_args.push(by_who.clone());
            score.callv("increase_score", func_args);
        }

        score.rpc("increase_score", &[by_who.clone()]);

        self.do_explosion(owner);
    }
}
