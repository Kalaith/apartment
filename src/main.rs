use macroquad::prelude::*;

mod game;
mod state;
mod building;
mod tenant;
mod economy;
mod simulation;
mod ui;
mod data;

mod assets;
mod save;

// Phase 3 modules
mod city;
mod consequences;
mod narrative;
mod util;

use game::Game;

fn window_conf() -> Conf {
    Conf {
        window_title: "Apartment".to_owned(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;

    loop {
        clear_background(Color::from_rgba(30, 30, 35, 255));
        game.update();
        game.draw();
        next_frame().await;
    }
}
