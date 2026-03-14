use crate::components::{
    Asteroid, Boss, BulletStyle, Difficulty, EnemyShip, EnemyType, Engine, Explosion, LootItem,
    LootType, MenuItem, Mission, Ship,
};
use crate::game::{upgrade_definition, Game, UPGRADABLE_IDS};
use crate::resources::Resources;
use crate::systems::load_score;
use macroquad::prelude::*;
use macroquad::rand::gen_range;

pub fn draw_text_centered(text: &str, y_offset: f32, size: u16, color: Color, res: &Resources) {
    let font = res.font.as_ref();
    let params = TextParams {
        font,
        font_size: size,
        color,
        ..Default::default()
    };

    let dims = measure_text(text, res.font.as_ref(), size, 1.0);
    draw_text_ex(
        text,
        screen_width() / 2.0 - dims.width / 2.0,
        screen_height() / 2.0 - dims.height / 2.0 + y_offset,
        params,
    );
}

pub fn draw_text_with_font(text: &str, x: f32, y: f32, size: f32, color: Color, res: &Resources) {
    let font = res.font.as_ref();
    let font_size = size as u16;

    let has_cyrillic = text.chars().any(|c| matches!(c, '\u{0400}'..='\u{04FF}'));
    let font_scale = if has_cyrillic { 0.72 } else { 1.0 };

    let params = TextParams {
        font,
        font_size,
        color,
        font_scale,
        ..Default::default()
    };
    draw_text_ex(text, x, y, params);
}

pub fn draw_background(texture: &Texture2D) {
    let screen_w = screen_width();
    let screen_h = screen_height();
    let tex_w = texture.width();
    let tex_h = texture.height();

    let scale_x = screen_w / tex_w;
    let scale_y = screen_h / tex_h;
    let scale = scale_x.max(scale_y);

    let final_w = tex_w * scale;
    let final_h = tex_h * scale;

    let x = (screen_w - final_w) / 2.0;
    let y = (screen_h - final_h) / 2.0;

    draw_texture_ex(
        texture,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(final_w, final_h)),
            ..Default::default()
        },
    );
}

pub fn draw_ship(
    ship: &Ship,
    body_tex: &Texture2D,
    flame_tex: &Texture2D,
    shield_tex: Option<&Texture2D>,
) {
    let r_rad = ship.rotation.to_radians();

    draw_engine(&ship.engine, ship.pos, r_rad, flame_tex);

    let ship_size = 72.0;

    draw_texture_ex(
        body_tex,
        ship.pos.x - ship_size / 2.0,
        ship.pos.y - ship_size / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(ship_size, ship_size)),
            rotation: r_rad + std::f32::consts::FRAC_PI_2,
            ..Default::default()
        },
    );

    if ship.has_shield() {
        if let Some(shield_texture) = shield_tex {
            let shield_size = ship_size * 1.8;
            let hp_ratio = ship.shield_hp / ship.shield_max_hp;
            let alpha = (hp_ratio * 0.7 + 0.3).min(1.0);
            let shield_color = Color::new(1.0, 1.0, 1.0, alpha);

            draw_texture_ex(
                shield_texture,
                ship.pos.x - shield_size / 2.0,
                ship.pos.y - shield_size / 2.0,
                shield_color,
                DrawTextureParams {
                    dest_size: Some(vec2(shield_size, shield_size)),
                    ..Default::default()
                },
            );
        }
    }
}

pub fn draw_engine(engine: &Engine, ship_pos: Vec2, ship_rotation_rad: f32, texture: &Texture2D) {
    if engine.current_thrust <= 0.05 {
        return;
    }

    let dir_vec = vec2(ship_rotation_rad.cos(), ship_rotation_rad.sin());

    let max_flame_w = 22.0;
    let max_flame_h = 52.0;

    let current_w = max_flame_w * engine.current_thrust;
    let flicker = gen_range(-3.0, 3.0) * engine.current_thrust;
    let current_h = max_flame_h * engine.current_thrust + flicker;

    let flame_pos = ship_pos - (dir_vec * engine.offset);

    draw_texture_ex(
        texture,
        flame_pos.x - current_w / 2.0,
        flame_pos.y - current_h / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(current_w, current_h)),
            rotation: ship_rotation_rad + std::f32::consts::FRAC_PI_2,
            ..Default::default()
        },
    );
}

const ENEMY_HP_BAR_HEIGHT: f32 = 5.0;
const ENEMY_HP_BAR_GAP: f32 = 5.0;
const ENEMY_HP_BAR_BG_COLOR: Color = Color::new(0.25, 0.0, 0.0, 0.9);

pub fn draw_enemy(enemy: &EnemyShip, res: &Resources) {
    let size = match enemy.enemy_type {
        EnemyType::Regular => vec2(60.0, 60.0),
        EnemyType::Kamikaze => vec2(45.0, 45.0),
    };
    let texture = match enemy.enemy_type {
        EnemyType::Regular => &res.enemy_small,
        EnemyType::Kamikaze => &res.enemy_kamikaze,
    };

    let bar_width = size.x;
    let bar_left = enemy.pos.x - bar_width / 2.0;
    let bar_top = enemy.pos.y - size.y / 2.0 - ENEMY_HP_BAR_GAP - ENEMY_HP_BAR_HEIGHT;

    draw_rectangle(
        bar_left,
        bar_top,
        bar_width,
        ENEMY_HP_BAR_HEIGHT,
        ENEMY_HP_BAR_BG_COLOR,
    );

    let ratio = (enemy.health / enemy.max_health).clamp(0.0, 1.0);
    let fill_width = bar_width * ratio;
    if fill_width > 0.0 {
        let hp_color = Color::new(1.0 - ratio, ratio, 0.0, 1.0);
        draw_rectangle(bar_left, bar_top, fill_width, ENEMY_HP_BAR_HEIGHT, hp_color);
    }

    draw_texture_ex(
        texture,
        enemy.pos.x - size.x / 2.0,
        enemy.pos.y - size.y / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(size),
            rotation: enemy.rotation + std::f32::consts::FRAC_PI_2,
            ..Default::default()
        },
    );
}

const BOSS_SIZE: f32 = 120.0;
const BOSS_HP_BAR_HEIGHT: f32 = 8.0;
const BOSS_HP_BAR_GAP: f32 = 8.0;

pub fn draw_boss(boss: &Boss, res: &Resources) {
    let size = vec2(BOSS_SIZE, BOSS_SIZE);
    let texture = &res.boss_1;

    let bar_width = size.x * 1.2;
    let bar_left = boss.pos.x - bar_width / 2.0;
    let bar_top = boss.pos.y - size.y / 2.0 - BOSS_HP_BAR_GAP - BOSS_HP_BAR_HEIGHT;

    draw_rectangle(
        bar_left,
        bar_top,
        bar_width,
        BOSS_HP_BAR_HEIGHT,
        ENEMY_HP_BAR_BG_COLOR,
    );

    let ratio = (boss.health / boss.max_health).clamp(0.0, 1.0);
    let fill_width = bar_width * ratio;
    if fill_width > 0.0 {
        let hp_color = Color::new(1.0 - ratio, ratio, 0.0, 1.0);
        draw_rectangle(bar_left, bar_top, fill_width, BOSS_HP_BAR_HEIGHT, hp_color);
    }

    draw_texture_ex(
        texture,
        boss.pos.x - size.x / 2.0,
        boss.pos.y - size.y / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(size),
            rotation: boss.rotation,
            ..Default::default()
        },
    );
}

pub fn draw_loot(item: &LootItem, res: &Resources) {
    let texture = match item.item_type {
        LootType::Scrap(_) => &res.loot_scrap,
        LootType::RareMetal(_) => &res.loot_rare,
        LootType::HealthPack(_) => &res.loot_health,
        LootType::RapidFireBoost => &res.loot_rapid_fire,
        LootType::BigBulletBoost => &res.loot_big_bullet,
        LootType::Shield(_) => &res.loot_shield,
    };

    let size = vec2(item.radius * 4.5, item.radius * 4.5);
    draw_texture_ex(
        texture,
        item.pos.x - size.x / 2.0,
        item.pos.y - size.y / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(size),
            rotation: item.rotation,
            ..Default::default()
        },
    );
}

pub fn draw_asteroid(asteroid: &Asteroid, res: &Resources) {
    let texture = if asteroid.is_rare {
        &res.rare_asteroid
    } else {
        &res.asteroid
    };

    let size = vec2(asteroid.radius * 2.0, asteroid.radius * 2.0);
    draw_texture_ex(
        texture,
        asteroid.pos.x - size.x / 2.0,
        asteroid.pos.y - size.y / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(size),
            ..Default::default()
        },
    );
}

pub fn draw_explosion(expl: &Explosion, res: &Resources) {
    let texture = &res.explosion;

    let frame_width = texture.width() / expl.max_frames as f32;
    let frame_height = texture.height();

    let source_rect = Rect::new(
        expl.frame as f32 * frame_width,
        0.0,
        frame_width,
        frame_height,
    );

    let draw_size = vec2(frame_width * expl.scale, frame_height * expl.scale);

    draw_texture_ex(
        texture,
        expl.pos.x - draw_size.x / 2.0,
        expl.pos.y - draw_size.y / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(draw_size),
            source: Some(source_rect),
            ..Default::default()
        },
    );
}

pub fn draw_enemy_direction_hint(from_left: bool, spawn_y: f32) {
    let t = get_time() as f32;
    let blink = 0.4 + 0.6 * (t * 2.5).sin().mul_add(0.5, 0.5);
    let color = Color::new(0.2, 1.0, 0.4, blink);

    let w = screen_width();
    let h = screen_height();
    let margin = 50.0;
    let cy = spawn_y.clamp(margin, h - margin);
    let edge_x = if from_left { 28.0 } else { w - 28.0 };
    let dir: f32 = if from_left { 1.0 } else { -1.0 };

    let arc_radii = [10.0, 18.0, 26.0, 34.0];
    let (angle_min, angle_max) = if from_left {
        (-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2)
    } else {
        (std::f32::consts::FRAC_PI_2, std::f32::consts::PI * 1.5)
    };
    for &r in &arc_radii {
        let n = (r * 0.8) as usize + 4;
        let mut prev: Option<Vec2> = None;
        for i in 0..=n {
            let angle = angle_min + (angle_max - angle_min) * (i as f32 / n as f32);
            let px = edge_x + r * angle.cos();
            let py = cy + r * angle.sin();
            let pt = Vec2::new(px, py);
            if let Some(p) = prev {
                draw_line(p.x, p.y, pt.x, pt.y, 2.0, color);
            }
            prev = Some(pt);
        }
    }

    let arrow_w = 10.0;
    let arrow_h = 14.0;
    let tip_x = edge_x + dir * (arrow_w + 8.0);
    let base_x = edge_x + dir * 8.0;
    let left_y = cy - arrow_h / 2.0;
    let right_y = cy + arrow_h / 2.0;
    let (tip, base_left, base_right) = if from_left {
        (
            Vec2::new(tip_x, cy),
            Vec2::new(base_x, left_y),
            Vec2::new(base_x, right_y),
        )
    } else {
        (
            Vec2::new(tip_x, cy),
            Vec2::new(base_x, right_y),
            Vec2::new(base_x, left_y),
        )
    };
    draw_triangle(tip, base_left, base_right, color);
}

pub fn render_game(game: &Game, resources: &Resources) {
    for item in &game.loot_items {
        draw_loot(item, resources);
    }

    for b in &game.bullets {
        let texture = match b.style {
            BulletStyle::Player => &resources.bullet,
            BulletStyle::Enemy => &resources.enemy_bullet,
        };

        let rotation = b.vel.y.atan2(b.vel.x) + std::f32::consts::FRAC_PI_2;
        let size = b.radius * 2.0;

        draw_texture_ex(
            texture,
            b.pos.x - size / 2.0,
            b.pos.y - size / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                rotation,
                ..Default::default()
            },
        );
    }

    for a in &game.asteroids {
        draw_asteroid(a, resources);
    }

    for e in &game.enemy_ships {
        draw_enemy(e, resources);
    }

    if let Some(ref boss) = game.boss {
        draw_boss(boss, resources);
    }

    for ex in &game.explosions {
        draw_explosion(ex, resources);
    }

    draw_ship(
        &game.ship,
        &resources.ship_body,
        &resources.ship_flame,
        Some(&resources.shield_active),
    );

    let mut status_text = format!(
        "{} {}  {} {:.0}/{:.0}",
        resources.lang.t("score"),
        game.score,
        resources.lang.t("hp"),
        game.ship.health,
        game.ship.max_health
    );
    if game.ship.has_shield() {
        status_text.push_str(&format!(
            "  {} {:.0}/{:.0}",
            resources.lang.t("shield"),
            game.ship.shield_hp,
            game.ship.shield_max_hp
        ));
    }
    draw_text_with_font(&status_text, 20.0, 30.0, 24.0, WHITE, resources);

    let status = if game.current_mission.is_boss_level {
        match &game.boss {
            Some(boss) => format!(
                "{} {:.0}/{:.0}",
                resources.lang.t("boss_hp"),
                boss.health,
                boss.max_health
            ),
            None => resources.lang.t("boss_defeated").to_string(),
        }
    } else {
        format!(
            "{} {}/{}  {} {}/{}  {} {}/{}",
            resources.lang.t("defeated"),
            game.mission_kills,
            game.current_mission.target_kills,
            resources.lang.t("rust"),
            game.mission_scrap_collected,
            game.current_mission.target_scrap,
            resources.lang.t("gold"),
            game.mission_rare_metal_collected,
            game.current_mission.target_rare_metal
        )
    };
    draw_text_with_font(
        &status,
        20.0,
        screen_height() - 30.0,
        24.0,
        WHITE,
        resources,
    );

    let inventory = format!(
        "{} {} {} | {} {}",
        resources.lang.t("resources"),
        resources.lang.t("rust"),
        game.ship.scrap,
        resources.lang.t("gold"),
        game.ship.rare_metal
    );
    draw_text_with_font(
        &inventory,
        20.0,
        screen_height() - 60.0,
        20.0,
        GRAY,
        resources,
    );

    if game.difficulty == Difficulty::Nebula {
        if let Some((from_left, spawn_y, _)) = game.pending_enemy_hint {
            draw_enemy_direction_hint(from_left, spawn_y);
        }
    }
}

pub fn render_menu(game: &Game, res: &Resources) {
    draw_background(&res.background);

    let time = get_time();
    let pulse = 1.0 + (time * 2.0).sin() as f32 * 0.05;

    let target_width = screen_width() * 0.3;
    let aspect_ratio = res.logo.height() / res.logo.width();
    let target_height = target_width * aspect_ratio;

    let logo_w = target_width * pulse;
    let logo_h = target_height * pulse;

    let logo_x = screen_width() / 2.0 - logo_w / 2.0;
    let logo_y = screen_height() / 2.0 - logo_h / 2.0 - 200.0;

    draw_texture_ex(
        &res.logo,
        logo_x,
        logo_y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(logo_w, logo_h)),
            ..Default::default()
        },
    );

    let base_y = logo_h / 2.0 - 50.0;
    let item_spacing = 60.0;

    let start_font_size = 32;
    let start_selected_font_size = 36;
    let other_font_size = 16;
    let other_selected_font_size = 18;

    let start_y = base_y + 40.0;
    let is_selected = game.menu_selection == MenuItem::Start;
    let start_color = if is_selected { YELLOW } else { WHITE };
    let start_size = if is_selected {
        start_selected_font_size
    } else {
        start_font_size
    };
    let start_prefix = if is_selected { "> " } else { "  " };
    draw_text_centered(
        &format!("{}{}", start_prefix, res.lang.t("menu_start")),
        start_y,
        start_size,
        start_color,
        res,
    );

    let diff_y = start_y + item_spacing;
    let is_selected = game.menu_selection == MenuItem::Difficulty;
    let diff_color = if is_selected { YELLOW } else { WHITE };
    let diff_size = if is_selected {
        other_selected_font_size
    } else {
        other_font_size
    };
    let diff_prefix = if is_selected { "> " } else { "  " };

    let diff_key = match game.difficulty {
        Difficulty::Nebula => "diff_nebula",
        Difficulty::Supernova => "diff_supernova",
        Difficulty::BlackHole => "diff_blackhole",
    };
    let diff_text = res.lang.t(diff_key);

    let final_diff_color = if is_selected {
        match game.difficulty {
            Difficulty::Nebula => GREEN,
            Difficulty::Supernova => YELLOW,
            Difficulty::BlackHole => RED,
        }
    } else {
        diff_color
    };

    draw_text_centered(
        &format!("{diff_prefix}< {diff_text} >"),
        diff_y,
        diff_size,
        final_diff_color,
        res,
    );

    let lang_y = diff_y + item_spacing;
    let is_selected = game.menu_selection == MenuItem::Language;
    let lang_color = if is_selected { YELLOW } else { WHITE };
    let lang_size = if is_selected {
        other_selected_font_size
    } else {
        other_font_size
    };
    let lang_prefix = if is_selected { "> " } else { "  " };

    let lang_text = match res.lang.current_lang {
        crate::localization::Language::English => res.lang.t("lang_english"),
        crate::localization::Language::Russian => res.lang.t("lang_russian"),
        crate::localization::Language::German => res.lang.t("lang_german"),
    };

    draw_text_centered(
        &format!("{lang_prefix}< {lang_text} >"),
        lang_y,
        lang_size,
        lang_color,
        res,
    );

    draw_text_centered(
        res.lang.t("menu_instructions"),
        base_y + 250.0,
        14,
        GRAY,
        res,
    );
}

pub fn render_briefing(mission: &Mission, res: &Resources) {
    draw_text_centered(
        &format!("{} {}", res.lang.t("mission"), mission.level_id),
        -100.0,
        32,
        ORANGE,
        res,
    );
    draw_text_centered(&mission.title, -50.0, 48, WHITE, res);
    draw_text_centered(&mission.description, 0.0, 20, GRAY, res);

    draw_text_centered(res.lang.t("objectives"), 20.0, 24, GRAY, res);

    let obj_text = if mission.is_boss_level {
        res.lang.t("obj_destroy_boss").to_string()
    } else {
        let mut objectives = vec![format!(
            "{} {} {}",
            res.lang.t("obj_destroy_prefix"),
            mission.target_kills,
            res.lang.t("obj_enemies")
        )];
        if mission.target_scrap > 0 {
            objectives.push(format!(
                "{} {} {}",
                res.lang.t("obj_scrap_prefix"),
                mission.target_scrap,
                res.lang.t("obj_rust_piles")
            ));
        }
        if mission.target_rare_metal > 0 {
            objectives.push(format!(
                "{} {} {}",
                res.lang.t("obj_gold_prefix"),
                mission.target_rare_metal,
                res.lang.t("obj_gold")
            ));
        }
        objectives.join("\n")
    };
    draw_text_centered(&obj_text, 70.0, 24, WHITE, res);

    draw_text_centered(res.lang.t("press_space"), 200.0, 24, GREEN, res);
}

pub fn render_mission_success(mission: &Mission, res: &Resources) {
    draw_text_centered(res.lang.t("mission_complete"), -50.0, 40, GREEN, res);
    draw_text_centered(
        &format!(
            "{} {} {}",
            res.lang.t("level_cleared_prefix"),
            mission.level_id,
            res.lang.t("level_cleared_suffix")
        ),
        10.0,
        24,
        WHITE,
        res,
    );
    draw_text_centered(res.lang.t("next_mission"), 100.0, 24, YELLOW, res);
}

pub fn render_upgrade_bay(game: &Game, res: &Resources) {
    draw_text_centered(res.lang.t("upgrade_bay_title"), -260.0, 40, ORANGE, res);
    draw_text_centered(res.lang.t("upgrade_bay_subtitle"), -220.0, 18, GRAY, res);

    let resources_text = format!(
        "{} {} {} | {} {}",
        res.lang.t("resources"),
        res.lang.t("rust"),
        game.ship.scrap,
        res.lang.t("gold"),
        game.ship.rare_metal
    );
    draw_text_centered(&resources_text, -175.0, 22, YELLOW, res);

    let start_y = screen_height() * 0.5 - 110.0;
    let line_height = 52.0;

    for (idx, id) in UPGRADABLE_IDS.iter().copied().enumerate() {
        let def = upgrade_definition(id);
        let level = game.upgrade_level(id);
        let max_level = def.max_level;
        let selected = game.upgrade_selection == idx;
        let line_color = if selected { YELLOW } else { WHITE };
        let prefix = if selected { "> " } else { "  " };

        let status_text = if level >= max_level {
            res.lang.t("upgrade_status_maxed").to_string()
        } else {
            let next_level = level + 1;
            let cost = crate::game::upgrade_cost(id, next_level);
            let can_buy = game.can_buy_upgrade(id);
            let buy_state = if can_buy {
                res.lang.t("upgrade_status_buy")
            } else {
                res.lang.t("upgrade_status_lack")
            };
            format!(
                "{}: {} {} {} {} ({})",
                res.lang.t("upgrade_cost"),
                res.lang.t("rust"),
                cost.scrap,
                res.lang.t("gold"),
                cost.rare_metal,
                buy_state
            )
        };

        let row_text = format!(
            "{prefix}{}  [{} {}/{}]  {}",
            res.lang.t(def.name_key),
            res.lang.t("upgrade_level"),
            level,
            max_level,
            status_text
        );

        draw_text_with_font(
            &row_text,
            screen_width() * 0.14,
            start_y + line_height * idx as f32,
            20.0,
            line_color,
            res,
        );

        let desc_text = res.lang.t(def.desc_key);
        draw_text_with_font(
            desc_text,
            screen_width() * 0.16,
            start_y + line_height * idx as f32 + 20.0,
            14.0,
            GRAY,
            res,
        );
    }

    let continue_idx = UPGRADABLE_IDS.len();
    let continue_selected = game.upgrade_selection == continue_idx;
    let continue_prefix = if continue_selected { "> " } else { "  " };
    let continue_color = if continue_selected { GREEN } else { WHITE };
    draw_text_with_font(
        &format!("{continue_prefix}{}", res.lang.t("upgrade_continue")),
        screen_width() * 0.14,
        start_y + line_height * continue_idx as f32 + 12.0,
        24.0,
        continue_color,
        res,
    );

    draw_text_centered(res.lang.t("upgrade_controls"), 285.0, 16, GRAY, res);
}

pub fn render_game_over(score: u32, res: &Resources) {
    let high_score = load_score().high_score;
    draw_text_centered(res.lang.t("game_over"), -40.0, 48, RED, res);
    draw_text_centered(
        &format!("{} {}", res.lang.t("final_score_prefix"), score),
        10.0,
        32,
        WHITE,
        res,
    );
    draw_text_centered(
        &format!("{} {}", res.lang.t("high_score"), high_score),
        60.0,
        24,
        YELLOW,
        res,
    );
}

pub fn render_pause(res: &Resources) {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.0, 0.0, 0.0, 0.7),
    );

    draw_text_centered(res.lang.t("paused"), -20.0, 48, YELLOW, res);
    draw_text_centered(res.lang.t("press_enter_resume"), 20.0, 24, WHITE, res);
    draw_text_centered(res.lang.t("press_esc"), 55.0, 24, WHITE, res);
}
