use crate::components::ShipKind;
use crate::localization::Localization;
use crate::model_sprite::ModelSprite;
use crate::ship_model::ShipModel;
use macroquad::audio::{load_sound, Sound};
use macroquad::prelude::*;
use std::path::Path;

#[cfg(target_os = "macos")]
pub(crate) fn macos_bundle_resources(executable: &Path) -> Option<std::path::PathBuf> {
    let macos = executable.parent()?;
    let contents = macos.parent()?;
    let bundle = contents.parent()?;
    if macos.ends_with("MacOS") && contents.ends_with("Contents") && bundle.extension()? == "app" {
        Some(contents.join("Resources"))
    } else {
        None
    }
}

pub struct Resources {
    pub logo: Texture2D,
    pub background: Texture2D,
    pub font: Option<Font>,
    pub lang: Localization,
    pub audio: AudioAssets,

    pub ship_body: Texture2D,
    ship_models: [Option<ShipModel>; ShipKind::ALL.len()],
    pub ship_flame: Texture2D,
    pub enemy_small: ModelSprite,
    pub enemy_kamikaze: ModelSprite,
    pub enemy_interceptor: ModelSprite,
    pub enemy_heavy: ModelSprite,
    pub enemy_guardian: ModelSprite,
    pub enemy_guardian_damaged: ModelSprite,
    pub bullet: Texture2D,
    pub enemy_bullet: ModelSprite,
    pub loot_scrap: ModelSprite,
    pub loot_rare: ModelSprite,
    pub loot_health: ModelSprite,
    pub loot_rapid_fire: ModelSprite,
    pub loot_big_bullet: ModelSprite,
    pub loot_magnetic_ammo: ModelSprite,
    pub loot_shield: ModelSprite,
    pub loot_invisibility: ModelSprite,
    pub shield_active: Texture2D,
    pub asteroid: ModelSprite,
    pub rare_asteroid: ModelSprite,
    pub explosion: Texture2D,
    pub boss_dreadnought: Texture2D,
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
    pub fn ship_model(&self, kind: ShipKind) -> Option<&ShipModel> {
        self.ship_models[kind as usize].as_ref()
    }

    pub fn selectable_ships(&self) -> impl Iterator<Item = ShipKind> + '_ {
        ShipKind::ALL
            .into_iter()
            .filter(|&kind| kind == ShipKind::Alpha || self.ship_model(kind).is_some())
    }

    // Async constructor that will load everything at once
    pub async fn new() -> Self {
        // Cargo builds use the project directory. Release packages resolve assets from the binary.
        if let Ok(executable) = std::env::current_exe() {
            let root = executable.parent().map(Path::to_path_buf);
            #[cfg(target_os = "macos")]
            let root = macos_bundle_resources(&executable).or(root);
            if let Some(root) = root
                .as_deref()
                .filter(|directory| directory.join("assets").is_dir())
                .and_then(Path::to_str)
            {
                set_pc_assets_folder(root);
            }
        }

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

        let ship_body = load_texture("assets/ship_body.png").await.unwrap();
        ship_body.set_filter(FilterMode::Nearest);
        let mut ship_models = [const { None }; ShipKind::ALL.len()];
        for kind in ShipKind::ALL {
            let path = match kind {
                ShipKind::Alpha => "assets/models/player_ship.glb",
                ShipKind::Delta => "assets/models/player_delta.glb",
                ShipKind::Specter => "assets/models/player_specter.glb",
                ShipKind::Vector => "assets/models/player_vector.glb",
            };
            ship_models[kind as usize] = ShipModel::load(path).await;
        }

        let ship_flame = load_texture("assets/ship_flame.png").await.unwrap();
        ship_flame.set_filter(FilterMode::Nearest);

        let overhead = vec3(0.0, 0.0, 2.0);
        let enemy_bullet = ModelSprite::load(
            "assets/enemy_bullet.png",
            "assets/models/enemy_projectile.glb",
            overhead,
            24,
        )
        .await;
        let enemy_small = ModelSprite::load(
            "assets/enemy.png",
            "assets/models/enemy_regular.glb",
            vec3(0.0, 0.85, 2.0),
            48,
        )
        .await;
        let enemy_kamikaze = ModelSprite::load(
            "assets/enemy_kamikaze.png",
            "assets/models/enemy_kamikaze.glb",
            overhead,
            40,
        )
        .await;
        let enemy_guardian = ModelSprite::load(
            "assets/boss_1.png",
            "assets/models/enemy_guardian.glb",
            overhead,
            72,
        )
        .await;
        let enemy_guardian_damaged = ModelSprite::load(
            "assets/boss_2.png",
            "assets/models/enemy_guardian_escalated.glb",
            overhead,
            72,
        )
        .await;
        let enemy_interceptor = enemy_small
            .with_model(
                "assets/models/enemy_interceptor.glb",
                vec3(0.0, 0.55, 2.0),
                40,
            )
            .await;
        // Tilt the heavy ship's view enough to show its recessed eye and three gun pods.
        let enemy_heavy = enemy_guardian
            .with_model("assets/models/enemy_heavy.glb", vec3(0.0, 1.15, 2.0), 56)
            .await;
        let boss_dreadnought = load_texture("assets/boss_dreadnought.png").await.unwrap();
        boss_dreadnought.set_filter(FilterMode::Linear);

        let loot_scrap = ModelSprite::load(
            "assets/loot/resources/rust_pile.png",
            "assets/models/loot_scrap.glb",
            vec3(0.0, 2.0, 1.8),
            48,
        )
        .await;
        let loot_rare = ModelSprite::load(
            "assets/loot/resources/gold.png",
            "assets/models/loot_gold.glb",
            overhead,
            48,
        )
        .await;
        // Show the top, left side, and identifying emblem as in the crate sprites.
        let crate_view = vec3(-1.5, 0.85, 2.0);
        let loot_health = ModelSprite::load(
            "assets/loot/health.png",
            "assets/models/loot_health.glb",
            crate_view,
            48,
        )
        .await;
        let loot_rapid_fire = ModelSprite::load(
            "assets/loot/energy.png",
            "assets/models/loot_rapid_fire.glb",
            crate_view,
            48,
        )
        .await;
        let loot_big_bullet = ModelSprite::load(
            "assets/loot/bigger-ammo.png",
            "assets/models/loot_big_bullet.glb",
            crate_view,
            48,
        )
        .await;
        let loot_shield = ModelSprite::load(
            "assets/loot/shield.png",
            "assets/models/loot_shield.glb",
            crate_view,
            48,
        )
        .await;
        let loot_magnetic_ammo = ModelSprite::load(
            "assets/loot/magnetic-ammo.png",
            "assets/models/magnetic_ammo.glb",
            crate_view,
            48,
        )
        .await;
        let loot_invisibility = ModelSprite::load(
            "assets/loot/mystery.png",
            "assets/models/invisibility.glb",
            vec3(-0.68, 0.55, 2.0),
            48,
        )
        .await;

        let shield_active = load_texture("assets/shield.png").await.unwrap();
        shield_active.set_filter(FilterMode::Nearest);

        let asteroid = ModelSprite::load(
            "assets/asteroid.png",
            "assets/models/asteroid.glb",
            vec3(0.0, 0.85, -2.0),
            40,
        )
        .await;
        let rare_asteroid = ModelSprite::load(
            "assets/rare_asteroid.png",
            "assets/models/rare_asteroid.glb",
            overhead,
            40,
        )
        .await;

        let explosion = load_texture("assets/explosion.png").await.unwrap();
        explosion.set_filter(FilterMode::Nearest);

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
            ship_models,
            ship_flame,
            enemy_small,
            enemy_kamikaze,
            enemy_interceptor,
            enemy_heavy,
            enemy_guardian,
            enemy_guardian_damaged,
            bullet,
            enemy_bullet,
            loot_scrap,
            loot_rare,
            loot_health,
            loot_rapid_fire,
            loot_big_bullet,
            loot_magnetic_ammo,
            loot_shield,
            loot_invisibility,
            shield_active,
            asteroid,
            rare_asteroid,
            explosion,
            boss_dreadnought,
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
