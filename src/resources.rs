use crate::localization::Localization;
use macroquad::audio::{load_sound, Sound};
use macroquad::prelude::*;

pub struct Resources {
    pub logo: Texture2D,
    pub background: Texture2D,
    pub font: Option<Font>,
    pub lang: Localization,
    pub audio: AudioAssets,

    pub ship_body: Texture2D,
    pub ship_flame: Texture2D,
    pub enemy_small: Texture2D,
    pub enemy_kamikaze: Texture2D,
    pub bullet: Texture2D,
    pub enemy_bullet: Texture2D,
    pub loot_scrap: Texture2D,
    pub loot_rare: Texture2D,
    pub loot_health: Texture2D,
    pub loot_rapid_fire: Texture2D,
    pub loot_big_bullet: Texture2D,
    pub loot_shield: Texture2D,
    pub shield_active: Texture2D,
    pub asteroid: Texture2D,
    pub rare_asteroid: Texture2D,
    pub explosion: Texture2D,
    pub boss_1: Texture2D,
}

#[allow(dead_code)]
pub struct AudioAssets {
    pub ui_move: Option<Sound>,
    pub ui_confirm: Option<Sound>,
    pub player_shot: Option<Sound>,
    pub enemy_shot: Option<Sound>,
    pub explosion_small: Option<Sound>,
    pub explosion_big: Option<Sound>,
    pub pickup_scrap: Option<Sound>,
    pub pickup_health: Option<Sound>,
    pub shield_on: Option<Sound>,
    pub shield_hit: Option<Sound>,
    pub ship_hit: Option<Sound>,
    pub mission_success: Option<Sound>,
    pub game_over: Option<Sound>,
    pub engine_loop: Option<Sound>,
    pub gameplay_loop: Option<Sound>,
    pub menu_loop: Option<Sound>,
}

impl Resources {
    // Async constructor that will load everything at once
    pub async fn new() -> Self {
        let logo = load_texture("assets/logo.png").await.unwrap();
        logo.set_filter(FilterMode::Nearest);

        // Try to load font, use None if it fails (will fall back to default font in draw_text_ex)
        let font = load_ttf_font("assets/Press_Start_2P/PressStart2P-Regular.ttf")
            .await
            .ok();

        let background = load_texture("assets/space_bg.png").await.unwrap();
        background.set_filter(FilterMode::Nearest);

        let bullet: Texture2D = load_texture("assets/bullet.png").await.unwrap();
        bullet.set_filter(FilterMode::Nearest);

        let enemy_bullet: Texture2D = load_texture("assets/enemy_bullet.png").await.unwrap();
        enemy_bullet.set_filter(FilterMode::Nearest);

        let ship_body = load_texture("assets/ship_body.png").await.unwrap();
        ship_body.set_filter(FilterMode::Nearest);

        let ship_flame = load_texture("assets/ship_flame.png").await.unwrap();
        ship_flame.set_filter(FilterMode::Nearest);

        let enemy_small = load_texture("assets/enemy.png").await.unwrap();
        enemy_small.set_filter(FilterMode::Nearest);

        let enemy_kamikaze = load_texture("assets/enemy_kamikaze.png").await.unwrap();
        enemy_kamikaze.set_filter(FilterMode::Nearest);

        let loot_scrap = load_texture("assets/loot/resources/rust_pile.png")
            .await
            .unwrap();
        let loot_rare = load_texture("assets/loot/resources/gold.png")
            .await
            .unwrap();
        let loot_health = load_texture("assets/loot/health.png").await.unwrap();
        let loot_rapid_fire = load_texture("assets/loot/energy.png").await.unwrap();
        let loot_big_bullet = load_texture("assets/loot/bigger-ammo.png").await.unwrap();
        let loot_shield = load_texture("assets/loot/shield.png").await.unwrap();
        let shield_active = load_texture("assets/shield.png").await.unwrap();
        shield_active.set_filter(FilterMode::Nearest);

        let asteroid = load_texture("assets/asteroid.png").await.unwrap();
        asteroid.set_filter(FilterMode::Nearest);

        let rare_asteroid = load_texture("assets/rare_asteroid.png").await.unwrap();
        rare_asteroid.set_filter(FilterMode::Nearest);

        let explosion = load_texture("assets/explosion.png").await.unwrap();
        explosion.set_filter(FilterMode::Nearest);

        let boss_1 = load_texture("assets/boss_1.png").await.unwrap();
        boss_1.set_filter(FilterMode::Nearest);

        let audio = AudioAssets {
            ui_move: load_audio_asset("assets/audio/sfx/ui_move").await,
            ui_confirm: load_audio_asset("assets/audio/sfx/ui_confirm").await,
            player_shot: load_audio_asset("assets/audio/sfx/player_shot").await,
            enemy_shot: load_audio_asset("assets/audio/sfx/enemy_shot").await,
            explosion_small: load_audio_asset("assets/audio/sfx/explosion_small").await,
            explosion_big: load_audio_asset("assets/audio/sfx/explosion_big").await,
            pickup_scrap: load_audio_asset("assets/audio/sfx/pickup_scrap").await,
            pickup_health: load_audio_asset("assets/audio/sfx/pickup_health").await,
            shield_on: load_audio_asset("assets/audio/sfx/shield_on").await,
            shield_hit: load_audio_asset("assets/audio/sfx/shield_hit").await,
            ship_hit: load_audio_asset("assets/audio/sfx/ship_hit").await,
            mission_success: load_audio_asset("assets/audio/sfx/mission_success").await,
            game_over: load_audio_asset("assets/audio/sfx/game_over").await,
            engine_loop: load_audio_asset("assets/audio/sfx/engine_loop").await,
            gameplay_loop: load_audio_asset("assets/audio/music/gameplay_loop").await,
            menu_loop: load_audio_asset("assets/audio/music/menu_loop").await,
        };
        let loaded_audio_count = [
            audio.ui_move.is_some(),
            audio.ui_confirm.is_some(),
            audio.player_shot.is_some(),
            audio.enemy_shot.is_some(),
            audio.explosion_small.is_some(),
            audio.explosion_big.is_some(),
            audio.pickup_scrap.is_some(),
            audio.pickup_health.is_some(),
            audio.shield_on.is_some(),
            audio.shield_hit.is_some(),
            audio.ship_hit.is_some(),
            audio.mission_success.is_some(),
            audio.game_over.is_some(),
            audio.engine_loop.is_some(),
            audio.gameplay_loop.is_some(),
            audio.menu_loop.is_some(),
        ]
        .iter()
        .filter(|loaded| **loaded)
        .count();
        if cfg!(debug_assertions) {
            println!("audio assets loaded: {loaded_audio_count}/16");
        }

        Self {
            logo,
            background,
            font,
            lang: Localization::new(),
            audio,
            ship_body,
            ship_flame,
            enemy_small,
            enemy_kamikaze,
            bullet,
            enemy_bullet,
            loot_scrap,
            loot_rare,
            loot_health,
            loot_rapid_fire,
            loot_big_bullet,
            loot_shield,
            shield_active,
            asteroid,
            rare_asteroid,
            explosion,
            boss_1,
        }
    }
}

async fn load_audio_asset(base_path: &str) -> Option<Sound> {
    let mut last_error = None;

    for extension in ["ogg", "wav"] {
        let path = format!("{base_path}.{extension}");
        match load_sound(&path).await {
            Ok(sound) => return Some(sound),
            Err(err) => last_error = Some(err),
        }
    }

    if cfg!(debug_assertions) {
        if let Some(err) = last_error {
            eprintln!("audio asset missing/unreadable for '{base_path}': {err}");
        }
    }

    None
}
