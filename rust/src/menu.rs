use godot::classes::{Control, IControl};
use godot::prelude::*;

/// Title screen; buttons are wired to these methods in menu.tscn.
#[derive(GodotClass)]
#[class(init, base=Control)]
pub struct MainMenu {
    base: Base<Control>,
}

#[godot_api]
impl MainMenu {
    #[func]
    fn on_start_pressed(&mut self) {
        let _ = self
            .base()
            .get_tree()
            .change_scene_to_file("res://scenes/main.tscn");
    }

    #[func]
    fn on_quit_pressed(&mut self) {
        self.base().get_tree().quit();
    }
}

#[godot_api]
impl IControl for MainMenu {}
