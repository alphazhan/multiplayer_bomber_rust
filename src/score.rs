use crate::utils;
use gdnative::api::*;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(HBoxContainer)]
pub struct Score;

#[methods]
impl Score {
    fn new(_owner: TRef<HBoxContainer>) -> Self {
        Score
    }

    #[export]
    fn _ready(&mut self, owner: TRef<HBoxContainer>) {
        self.get_winner(owner).hide();

        // Exit Button
        let exit_button = owner.get_node("../Winner/ExitGame").unwrap();
        let exit_button = unsafe { exit_button.assume_safe() };
        let exit_button = exit_button.cast::<Button>().unwrap();
        //

        let status_exit_button = exit_button.connect(
            "pressed",
            owner,
            "_on_exit_game_button_pressed",
            VariantArray::new_shared(),
            0,
        );

        if let Err(e) = status_exit_button {
            godot_error!(
                "`Score` => GodotError at `exit_button.connect` function: {}",
                e
            );
        }

        owner.set_process(true);
    }

    #[export]
    fn _process(&self, owner: TRef<HBoxContainer>, _delta: f64) {
        let rocks_left = self.get_rocks(owner).get_child_count();

        if rocks_left == 0 {
            let mut winner_name = String::from("");
            let mut winner_score = 0;

            for p_label in owner.get_children().iter() {
                let p_label = p_label.try_to_object::<Label>().unwrap();
                let p_label = unsafe { p_label.assume_safe() };

                let p_id = p_label.name();
                let p_score: i32 = p_label
                    .text()
                    .to_string()
                    .split('\n')
                    .collect::<Vec<&str>>()[1]
                    .parse()
                    .unwrap();

                println!("p_id: {}, p_score: {}", p_id, p_score);

                if p_score > winner_score {
                    winner_name = p_label
                        .text()
                        .to_string()
                        .split('\n')
                        .collect::<Vec<&str>>()[0]
                        .parse()
                        .unwrap();

                    winner_score = p_score;
                }
            }

            self.get_winner(owner)
                .set_text(format!("THE WINNER IS:\n{}", winner_name));
            self.get_winner(owner).show();
        }
    }

    /// Remote (Sync)
    #[export(rpc = "remote")]
    fn increase_score(&self, owner: TRef<HBoxContainer>, for_who: Variant) {
        // Player label
        let p_label = owner.get_node(for_who.to_string()).unwrap();
        let p_label = unsafe { p_label.assume_safe() };
        let p_label = p_label.cast::<Label>().unwrap();
        //

        let p_score: i32 = p_label
            .text()
            .to_string()
            .split('\n')
            .collect::<Vec<&str>>()[1]
            .parse()
            .unwrap();

        let p_name = unsafe {
            utils::get_root(owner.as_ref())
                .get_node(format!("World/Players/{}/nickname", for_who.to_string()))
                .unwrap()
                .assume_safe()
                .cast::<Label>()
                .unwrap()
                .text()
        };

        p_label.set_text(format!("{}\n{}", p_name, p_score + 1));
    }

    #[export]
    fn add_player(&self, owner: TRef<HBoxContainer>, id: Variant, new_player_name: Variant) {
        // Label
        let label = Label::new().into_shared();
        let label = unsafe { label.assume_safe() };
        //

        label.set_name(id.to_string());
        label.set_align(Label::ALIGN_CENTER);
        label.set_text(new_player_name.to_string() + "\n0");
        label.set_h_size_flags(Control::SIZE_EXPAND_FILL);

        // Font
        let font = DynamicFont::new().into_shared();
        let font = unsafe { font.assume_safe() };
        //

        font.set_size(18);

        let font_data = ResourceLoader::godot_singleton()
            .load("res://res/fonts/Ubuntu-Medium.ttf", "PackedScene", false)
            .unwrap();
        let font_data = unsafe { font_data.assume_safe() };
        let font_data = font_data.cast::<DynamicFontData>().unwrap();

        font.set_font_data(font_data);
        label.add_font_override("font", font);
        owner.add_child(label, true);
    }

    #[export]
    unsafe fn _on_exit_game_button_pressed(&self, owner: TRef<HBoxContainer>) {
        utils::get_gamestate_singleton(owner.as_ref())
            .callv("end_game", VariantArray::new_shared());
    }

    fn get_winner(&self, owner: TRef<HBoxContainer>) -> TRef<Label> {
        let world = unsafe { utils::get_world(owner.as_ref()) };

        let winner = world.get_node("Winner").unwrap();
        let winner = unsafe { winner.assume_safe() };
        winner.cast::<Label>().unwrap()
    }

    fn get_rocks(&self, owner: TRef<HBoxContainer>) -> TRef<Node2D> {
        let world = unsafe { utils::get_world(owner.as_ref()) };

        let rocks = world.get_node("Rocks").unwrap();
        let rocks = unsafe { rocks.assume_safe() };
        rocks.cast::<Node2D>().unwrap()
    }
}
