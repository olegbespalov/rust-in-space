use crate::components::ShipKind;
use crate::localization::{Language, Localization};
use crate::model_sprite::ModelSprite;
use crate::ship_model::ShipModel;
use macroquad::audio::{load_sound, Sound};
use macroquad::prelude::*;
use std::cell::OnceCell;
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
    pub shop_background: Texture2D,
    pub font: Option<Font>,
    japanese_font: OnceCell<Font>,
    chinese_font: OnceCell<Font>,
    korean_font: OnceCell<Font>,
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
    pub loot_medium_scrap: ModelSprite,
    pub loot_large_scrap: ModelSprite,
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
    pub fn text_font(&self, text: &str) -> Option<&Font> {
        let language = match self.lang.current_lang {
            language @ (Language::Japanese | Language::Chinese | Language::Korean) => language,
            _ if text.chars().any(|c| matches!(c, '\u{3040}'..='\u{30ff}' | '\u{ff66}'..='\u{ff9f}')) => {
                Language::Japanese
            }
            _ if text.chars().any(|c| matches!(c, '\u{1100}'..='\u{11ff}' | '\u{3130}'..='\u{318f}' | '\u{ac00}'..='\u{d7ff}')) => {
                Language::Korean
            }
            _ if text.chars().any(|c| matches!(c, '\u{3000}'..='\u{303f}' | '\u{3400}'..='\u{9fff}' | '\u{f900}'..='\u{faff}')) => {
                Language::Chinese
            }
            _ => return self.font.as_ref(),
        };
        let (font, bytes) = match language {
            Language::Japanese => (
                &self.japanese_font,
                include_bytes!("localization/fonts/RustInSpaceCJKJP-Regular.otf").as_slice(),
            ),
            Language::Chinese => (
                &self.chinese_font,
                include_bytes!("localization/fonts/RustInSpaceCJKSC-Regular.otf").as_slice(),
            ),
            Language::Korean => (
                &self.korean_font,
                include_bytes!("localization/fonts/RustInSpaceCJKKR-Regular.otf").as_slice(),
            ),
            _ => return self.font.as_ref(),
        };
        Some(font.get_or_init(|| load_ttf_font_from_bytes(bytes).expect("valid bundled CJK font")))
    }

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
        let shop_background = load_texture("assets/mercenary_port.png").await.unwrap();
        shop_background.set_filter(FilterMode::Linear);

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

        let enemy_bullet = ModelSprite::load("enemy_projectile", "assets/enemy_bullet.png").await;
        let enemy_small = ModelSprite::load("enemy_regular", "assets/enemy.png").await;
        let enemy_kamikaze = ModelSprite::load("enemy_kamikaze", "assets/enemy_kamikaze.png").await;
        let enemy_guardian = ModelSprite::load("enemy_guardian", "assets/boss_1.png").await;
        let enemy_guardian_damaged =
            ModelSprite::load("enemy_guardian_escalated", "assets/boss_2.png").await;
        let enemy_interceptor = ModelSprite::load("enemy_interceptor", "assets/enemy.png").await;
        let enemy_heavy = ModelSprite::load("enemy_heavy", "assets/boss_1.png").await;
        let boss_dreadnought = load_texture("assets/boss_dreadnought.png").await.unwrap();
        boss_dreadnought.set_filter(FilterMode::Linear);

        let loot_scrap =
            ModelSprite::load("loot_scrap", "assets/loot/resources/rust_pile.png").await;
        let loot_medium_scrap =
            ModelSprite::load("loot_scrap_medium", "assets/loot/resources/rust_pile.png").await;
        let loot_large_scrap =
            ModelSprite::load("loot_scrap_large", "assets/loot/resources/rust_pile.png").await;
        let loot_rare = ModelSprite::load("loot_gold", "assets/loot/resources/gold.png").await;
        let loot_health = ModelSprite::load("loot_health", "assets/loot/health.png").await;
        let loot_rapid_fire = ModelSprite::load("loot_rapid_fire", "assets/loot/energy.png").await;
        let loot_big_bullet =
            ModelSprite::load("loot_big_bullet", "assets/loot/bigger-ammo.png").await;
        let loot_shield = ModelSprite::load("loot_shield", "assets/loot/shield.png").await;
        let loot_magnetic_ammo =
            ModelSprite::load("magnetic_ammo", "assets/loot/magnetic-ammo.png").await;
        let loot_invisibility = ModelSprite::load("invisibility", "assets/loot/mystery.png").await;

        let shield_active = load_texture("assets/shield.png").await.unwrap();
        shield_active.set_filter(FilterMode::Nearest);

        let asteroid = ModelSprite::load("asteroid", "assets/asteroid.png").await;
        let rare_asteroid = ModelSprite::load("rare_asteroid", "assets/rare_asteroid.png").await;

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
            shop_background,
            font,
            japanese_font: OnceCell::new(),
            chinese_font: OnceCell::new(),
            korean_font: OnceCell::new(),
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
            loot_medium_scrap,
            loot_large_scrap,
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
