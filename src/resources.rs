use crate::localization::Localization;
use macroquad::prelude::*;

pub struct Resources {
    pub logo: Texture2D,
    pub background: Texture2D,
    pub font: Option<Font>,
    pub lang: Localization,

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

        Self {
            logo,
            background,
            font,
            lang: Localization::new(),
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
        }
    }
}
