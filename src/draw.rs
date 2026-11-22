use crate::components::{
    Asteroid, AudioSettings, Boss, BossPhase, BulletStyle, Difficulty, EnemyShip, EnemyType,
    Engine, Explosion, LootItem, LootType, MenuItem, Mission, MissionObjective, ShieldKind, Ship,
    ShipKind,
};
use crate::game::constants::{
    ENEMY_SMALL_RADIUS, HEAVY_RADIUS, INTERCEPTOR_RADIUS, KAMIKAZE_RADIUS,
};
use crate::game::{upgrade_definition, Game, UPGRADABLE_IDS};
use crate::resources::Resources;
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

const SHIP_VISUAL_SCALE: f32 = 1.25;
const INVISIBLE_TINT: Color = Color::new(0.65, 0.9, 1.0, 0.3);

pub fn draw_ship(ship: &Ship, resources: &Resources) {
    let r_rad = ship.rotation.to_radians();
    let hull_rotation = r_rad + std::f32::consts::FRAC_PI_2;

    let tint = if ship.is_invisible() {
        INVISIBLE_TINT
    } else {
        draw_engine(&ship.engine, ship.pos, r_rad, &resources.ship_flame);
        WHITE
    };

    let ship_size = 72.0 * SHIP_VISUAL_SCALE;
    draw_ship_hull(
        ship.kind,
        ship.pos,
        ship_size,
        hull_rotation,
        ship.engine.current_thrust,
        tint,
        resources,
    );

    if ship.has_shield() {
        draw_active_shield(
            ship.pos,
            ship_size,
            ship.shield_kind,
            ship.shield_hp / ship.shield_max_hp,
            tint,
            &resources.shield_active,
        );
    }
}

fn draw_active_shield(
    pos: Vec2,
    hull_size: f32,
    kind: ShieldKind,
    hp_ratio: f32,
    tint: Color,
    texture: &Texture2D,
) {
    let size = hull_size * 1.8;
    let alpha = (hp_ratio * 0.7 + 0.3).min(1.0) * tint.a;
    let color = match kind {
        ShieldKind::Standard => Color::new(tint.r, tint.g, tint.b, alpha),
        ShieldKind::Golden => Color::new(GOLD.r, GOLD.g, GOLD.b, alpha),
    };
    draw_texture_ex(
        texture,
        pos.x - size / 2.0,
        pos.y - size / 2.0,
        color,
        DrawTextureParams {
            dest_size: Some(Vec2::splat(size)),
            ..Default::default()
        },
    );
}

fn draw_ship_hull(
    kind: ShipKind,
    position: Vec2,
    size: f32,
    rotation: f32,
    thrust: f32,
    tint: Color,
    res: &Resources,
) {
    if let Some(model) = res.ship_model(kind) {
        model.draw(position, size, rotation, thrust, tint);
    } else {
        draw_texture_ex(
            &res.ship_body,
            position.x - size / 2.0,
            position.y - size / 2.0,
            tint,
            DrawTextureParams {
                dest_size: Some(Vec2::splat(size)),
                rotation,
                ..Default::default()
            },
        );
    }
}

fn draw_engine(engine: &Engine, ship_pos: Vec2, ship_rotation_rad: f32, texture: &Texture2D) {
    if engine.current_thrust <= 0.05 {
        return;
    }

    let dir_vec = vec2(ship_rotation_rad.cos(), ship_rotation_rad.sin());

    let max_flame_w = 22.0;
    let max_flame_h = 52.0;

    let current_w = max_flame_w * engine.current_thrust * SHIP_VISUAL_SCALE;
    let flicker = gen_range(-3.0, 3.0) * engine.current_thrust;
    let current_h = (max_flame_h * engine.current_thrust + flicker) * SHIP_VISUAL_SCALE;

    let flame_pos = ship_pos - (dir_vec * engine.offset * SHIP_VISUAL_SCALE);

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

fn health_bar_top(center_y: f32, radius: f32, gap: f32, height: f32) -> f32 {
    let above = center_y - radius - gap - height;
    if above < 0.0 {
        center_y + radius + gap
    } else {
        above
    }
}

fn health_bar_left(center_x: f32, width: f32) -> f32 {
    (center_x - width / 2.0).clamp(0.0, (screen_width() - width).max(0.0))
}

pub fn draw_enemy(enemy: &EnemyShip, res: &Resources) {
    let (radius, sprite) = match enemy.enemy_type {
        EnemyType::Regular => (ENEMY_SMALL_RADIUS, &res.enemy_small),
        EnemyType::Kamikaze => (KAMIKAZE_RADIUS, &res.enemy_kamikaze),
        EnemyType::Interceptor => (INTERCEPTOR_RADIUS, &res.enemy_interceptor),
        EnemyType::Heavy => (HEAVY_RADIUS, &res.enemy_heavy),
    };
    let visual_radius = radius * SHIP_VISUAL_SCALE;
    let size = visual_radius * 2.0;

    let bar_left = health_bar_left(enemy.pos.x, size);
    let bar_top = health_bar_top(
        enemy.pos.y,
        visual_radius,
        ENEMY_HP_BAR_GAP,
        ENEMY_HP_BAR_HEIGHT,
    );

    let tint = if enemy.power_ups.is_invisible() {
        INVISIBLE_TINT
    } else {
        draw_enemy_health_bar(
            bar_left,
            bar_top,
            size,
            ENEMY_HP_BAR_HEIGHT,
            enemy.health / enemy.max_health,
        );
        WHITE
    };

    let rotation = enemy.rotation + std::f32::consts::FRAC_PI_2;
    sprite.draw_tinted(enemy.pos, size, rotation, tint);
    if let Some(shield) = &enemy.power_ups.shield {
        draw_active_shield(
            enemy.pos,
            size,
            shield.kind,
            shield.hp / shield.max_hp,
            tint,
            &res.shield_active,
        );
    }
}

const BOSS_SIZE: f32 = 120.0;
const BOSS_HP_BAR_HEIGHT: f32 = 8.0;
const BOSS_HP_BAR_GAP: f32 = 8.0;

pub fn draw_boss(boss: &Boss, res: &Resources) {
    let size = BOSS_SIZE * SHIP_VISUAL_SCALE;
    let bar_width = size * 1.2;
    let bar_left = health_bar_left(boss.pos.x, bar_width);
    let bar_top = health_bar_top(boss.pos.y, size / 2.0, BOSS_HP_BAR_GAP, BOSS_HP_BAR_HEIGHT);

    let tint = if boss.power_ups.is_invisible() {
        INVISIBLE_TINT
    } else {
        draw_enemy_health_bar(
            bar_left,
            bar_top,
            bar_width,
            BOSS_HP_BAR_HEIGHT,
            boss.health / boss.max_health,
        );
        WHITE
    };

    let model = match boss.phase {
        BossPhase::Opening => &res.boss_1,
        BossPhase::Escalated => &res.boss_2,
    };
    model.draw_tinted(boss.pos, size, boss.rotation, tint);
    if let Some(shield) = &boss.power_ups.shield {
        draw_active_shield(
            boss.pos,
            size,
            shield.kind,
            shield.hp / shield.max_hp,
            tint,
            &res.shield_active,
        );
    }
}

fn draw_enemy_health_bar(x: f32, y: f32, width: f32, height: f32, ratio: f32) {
    draw_rectangle(x, y, width, height, ENEMY_HP_BAR_BG_COLOR);
    let ratio = ratio.clamp(0.0, 1.0);
    if ratio > 0.0 {
        let hp_color = Color::new(1.0 - ratio, ratio, 0.0, 1.0);
        draw_rectangle(x, y, width * ratio, height, hp_color);
    }
}

pub fn draw_loot(item: &LootItem, res: &Resources) {
    let size = item.radius * 4.5;
    let sprite = match item.item_type {
        LootType::Scrap(_) => &res.loot_scrap,
        LootType::RareMetal(_) => &res.loot_rare,
        LootType::HealthPack(_) => &res.loot_health,
        LootType::RapidFireBoost => &res.loot_rapid_fire,
        LootType::BigBulletBoost => &res.loot_big_bullet,
        LootType::MagneticAmmo => &res.loot_magnetic_ammo,
        LootType::Shield(..) => &res.loot_shield,
        LootType::Invisibility | LootType::MysteryBox => &res.loot_invisibility,
    };

    sprite.draw(item.pos, size, item.rotation);
    if matches!(
        item.item_type,
        LootType::MysteryBox | LootType::Shield(ShieldKind::Golden, _)
    ) {
        draw_circle_lines(item.pos.x, item.pos.y, size * 0.5, 2.0, GOLD);
    }

    const TIMER_FONT_SIZE: u16 = 14;
    const TIMER_GAP: f32 = 4.0;
    let seconds = format!("{}", item.time_left.ceil() as u32);
    let width = measure_text(&seconds, res.font.as_ref(), TIMER_FONT_SIZE, 1.0).width;
    let right = item.pos.x + size * 0.5 + TIMER_GAP;
    let x = if right + width + TIMER_GAP <= screen_width() {
        right.max(TIMER_GAP)
    } else {
        item.pos.x - size * 0.5 - TIMER_GAP - width
    };
    let font_size = f32::from(TIMER_FONT_SIZE);
    let y = (item.pos.y + font_size / 3.0).clamp(font_size, (screen_height() - 1.0).max(font_size));
    let color = if item.time_left <= 3.0 { GOLD } else { WHITE };
    draw_text_with_font(&seconds, x + 1.0, y + 1.0, font_size, BLACK, res);
    draw_text_with_font(&seconds, x, y, font_size, color, res);
}

pub fn draw_asteroid(asteroid: &Asteroid, res: &Resources) {
    const ASTEROID_VISUAL_SCALE: f32 = 1.5;
    let size = asteroid.radius * 2.0 * ASTEROID_VISUAL_SCALE;
    let sprite = if asteroid.is_rare {
        &res.rare_asteroid
    } else {
        &res.asteroid
    };
    sprite.draw(asteroid.pos, size, 0.0);
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

fn draw_enemy_direction_hint(from_left: bool, spawn_y: f32, t: f32) {
    let blink = 0.4 + 0.6 * (t * 2.5).sin().mul_add(0.5, 0.5);
    let color = Color::new(0.2, 1.0, 0.4, blink);

    let w = screen_width();
    let h = screen_height();
    let margin = 50.0_f32.min(h / 2.0);
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
        let rotation = b.vel.y.atan2(b.vel.x) + std::f32::consts::FRAC_PI_2;
        let size = b.radius * 2.0;
        if b.style == BulletStyle::Enemy {
            resources.enemy_bullet.draw(b.pos, size, rotation);
            continue;
        }

        let tint = if b.homing {
            Color::from_rgba(210, 145, 255, 255)
        } else {
            WHITE
        };

        draw_texture_ex(
            &resources.bullet,
            b.pos.x - size / 2.0,
            b.pos.y - size / 2.0,
            tint,
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

    draw_ship(&game.ship, resources);

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
    if game.ship.is_invisible() {
        let cloak_text = format!(
            "{} {:.0}",
            resources.lang.t("invisibility"),
            game.ship.invisibility_timer.ceil()
        );
        draw_text_with_font(&cloak_text, 20.0, 60.0, 24.0, SKYBLUE, resources);
    }

    let objectives = &game.current_mission.objectives;
    for (row, &objective) in objectives.iter().enumerate() {
        let complete = game.mission_progress.meets(objective);
        let status = match objective {
            MissionObjective::DefeatBoss => match &game.boss {
                Some(boss) => {
                    let phase = match boss.phase {
                        BossPhase::Opening => 1,
                        BossPhase::Escalated => 2,
                    };
                    format!(
                        "{} {:.0}/{:.0}  {} {phase}/2",
                        resources.lang.t("boss_hp"),
                        boss.health.ceil(),
                        boss.max_health,
                        resources.lang.t("boss_phase")
                    )
                }
                None => resources.lang.t("boss_defeated").to_string(),
            },
            _ => format!(
                "{}  {}/{}",
                resources.lang.t(objective_label(objective)),
                game.mission_progress
                    .amount(objective)
                    .min(objective.target()),
                objective.target()
            ),
        };
        draw_text_with_font(
            &status,
            20.0,
            screen_height() - 30.0 - (objectives.len() - 1 - row) as f32 * 26.0,
            24.0,
            if complete { GREEN } else { WHITE },
            resources,
        );
    }

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
        screen_height() - 34.0 - objectives.len() as f32 * 26.0,
        20.0,
        GRAY,
        resources,
    );

    if let Some(boss) = &game.boss {
        if let Some(wave) = boss.pending_summon {
            for (from_left, y, _) in wave.arrivals(boss.phase) {
                draw_enemy_direction_hint(from_left, y, -wave.remaining);
            }
        }
    } else if game.difficulty == Difficulty::Nebula {
        if let Some((from_left, spawn_y, _)) = game.pending_enemy_hint {
            draw_enemy_direction_hint(from_left, spawn_y, get_time() as f32);
        }
    }
}

fn draw_menu_button(bounds: Rect, selected: bool) -> bool {
    let (mouse_x, mouse_y) = mouse_position();
    let hovered = bounds.contains(vec2(mouse_x, mouse_y));
    let rust = if hovered {
        Color::from_rgba(184, 91, 48, 255)
    } else {
        Color::from_rgba(151, 70, 39, 255)
    };
    let edge = if hovered || selected {
        Color::from_rgba(221, 153, 76, 255)
    } else {
        Color::from_rgba(106, 73, 54, 255)
    };
    let face = if hovered {
        Color::from_rgba(66, 53, 45, 255)
    } else {
        Color::from_rgba(45, 44, 43, 255)
    };
    let inset = 7.0_f32.min(bounds.w / 9.0).min(bounds.h / 5.0);

    draw_rectangle(
        bounds.x + 3.0,
        bounds.y + 4.0,
        bounds.w,
        bounds.h,
        Color::from_rgba(8, 10, 13, 190),
    );
    draw_rectangle(bounds.x, bounds.y, bounds.w, bounds.h, rust);
    draw_rectangle(
        bounds.x + inset,
        bounds.y + inset,
        (bounds.w - inset * 2.0).max(0.0),
        (bounds.h - inset * 2.0).max(0.0),
        face,
    );
    // Uneven rust streaks make the plate read as worn metal instead of a flat panel.
    let streak_y = bounds.y + inset * 0.48;
    draw_line(
        bounds.x + bounds.w * 0.22,
        streak_y,
        bounds.x + bounds.w * 0.38,
        streak_y - 1.0,
        2.0,
        Color::from_rgba(222, 136, 73, 190),
    );
    draw_line(
        bounds.x + bounds.w * 0.67,
        bounds.bottom() - inset * 0.46,
        bounds.x + bounds.w * 0.82,
        bounds.bottom() - inset * 0.46 + 1.0,
        2.0,
        Color::from_rgba(93, 48, 34, 220),
    );
    draw_line(
        bounds.x + inset + 2.0,
        bounds.y + inset + 2.0,
        bounds.right() - inset - 2.0,
        bounds.y + inset + 2.0,
        1.0,
        Color::from_rgba(195, 133, 82, 105),
    );
    draw_line(
        bounds.x + inset + 2.0,
        bounds.bottom() - inset - 2.0,
        bounds.right() - inset - 2.0,
        bounds.bottom() - inset - 2.0,
        1.0,
        Color::from_rgba(10, 12, 14, 150),
    );
    draw_rectangle_lines(bounds.x, bounds.y, bounds.w, bounds.h, 1.5, edge);

    if bounds.w > 72.0 && bounds.h > 28.0 {
        let radius = 2.5_f32.min(bounds.h / 10.0);
        for (x, y) in [
            (bounds.x + 7.0, bounds.y + 7.0),
            (bounds.right() - 7.0, bounds.y + 7.0),
            (bounds.x + 7.0, bounds.bottom() - 7.0),
            (bounds.right() - 7.0, bounds.bottom() - 7.0),
        ] {
            draw_circle(x, y, radius, Color::from_rgba(190, 125, 73, 220));
            draw_line(
                x - radius * 0.55,
                y,
                x + radius * 0.55,
                y,
                0.7,
                Color::from_rgba(48, 34, 28, 230),
            );
        }
    }

    hovered && is_mouse_button_pressed(MouseButton::Left)
}

fn draw_menu_label(text: &str, bounds: Rect, size: u16, color: Color, res: &Resources) {
    draw_menu_label_with_scale(text, bounds, size, color, 1.0, res);
}

fn draw_menu_label_with_scale(
    text: &str,
    bounds: Rect,
    size: u16,
    color: Color,
    max_scale: f32,
    res: &Resources,
) {
    let font = res.font.as_ref();
    let dims = measure_text(text, font, size, 1.0);
    let inner_x = 22.0_f32.min(bounds.w * 0.18);
    let inner_y = 10.0_f32.min(bounds.h * 0.24);
    let scale = ((bounds.w - inner_x * 2.0) / dims.width)
        .min((bounds.h - inner_y * 2.0) / dims.height)
        .min(max_scale)
        .clamp(0.0, 1.0);
    draw_text_ex(
        text,
        bounds.x + (bounds.w - dims.width * scale) / 2.0,
        bounds.y + (bounds.h - dims.height * scale) / 2.0 + dims.offset_y * scale,
        TextParams {
            font,
            font_size: size,
            font_scale: scale,
            color,
            ..Default::default()
        },
    );
}

fn menu_button(text: &str, bounds: Rect, size: u16, selected: bool, res: &Resources) -> bool {
    let clicked = draw_menu_button(bounds, selected);
    draw_menu_label_with_scale(text, bounds, size, WHITE, 0.84, res);
    clicked
}

fn centered_menu_button(text: &str, y_offset: f32, res: &Resources) -> bool {
    let width =
        (measure_text(text, res.font.as_ref(), 24, 1.0).width + 64.0).min(screen_width() * 0.94);
    let bounds = Rect::new(
        (screen_width() - width) / 2.0,
        screen_height() / 2.0 + y_offset - 26.0,
        width,
        52.0,
    );
    menu_button(text, bounds, 24, false, res)
}

#[derive(Clone, Copy)]
pub enum MenuClick {
    Activate(MenuItem),
    Select(MenuItem),
    DecreaseVolume(MenuItem),
    IncreaseVolume(MenuItem),
}

impl MenuClick {
    pub fn item(self) -> MenuItem {
        match self {
            Self::Activate(item)
            | Self::Select(item)
            | Self::DecreaseVolume(item)
            | Self::IncreaseVolume(item) => item,
        }
    }
}

pub fn render_menu(
    game: &Game,
    high_score: u32,
    settings: &AudioSettings,
    res: &Resources,
) -> Option<MenuClick> {
    let screen_w = screen_width();
    let screen_h = screen_height();
    let pulse = 1.0 + (get_time() * 2.0).sin() as f32 * 0.05;
    let aspect_ratio = res.logo.height() / res.logo.width();
    let logo_w = (screen_w * 0.3).min(screen_h * 0.28 / aspect_ratio) * pulse;
    let logo_h = logo_w * aspect_ratio;
    draw_texture_ex(
        &res.logo,
        (screen_w - logo_w) / 2.0,
        screen_h * 0.18 - logo_h / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(logo_w, logo_h)),
            ..Default::default()
        },
    );

    let start_y = screen_h * 0.39;
    let row_step = (screen_h * 0.075).min(46.0);
    let row_width = (screen_w * 0.94).min(420.0);
    let row_x = (screen_w - row_width) / 2.0;
    draw_text_centered(
        &format!("{} {}", res.lang.t("high_score"), high_score),
        screen_h * 0.35 - screen_h / 2.0,
        18,
        YELLOW,
        res,
    );

    let difficulty = match game.difficulty {
        Difficulty::Nebula => "diff_nebula",
        Difficulty::Supernova => "diff_supernova",
        Difficulty::BlackHole => "diff_blackhole",
    };
    let language = match res.lang.current_lang {
        crate::localization::Language::English => "lang_english",
        crate::localization::Language::Russian => "lang_russian",
        crate::localization::Language::German => "lang_german",
    };
    let audio_status = if settings.audio_muted {
        "audio_off"
    } else {
        "audio_on"
    };

    let rows = [
        (MenuItem::Start, "menu_start", None),
        (
            MenuItem::Difficulty,
            "menu_difficulty",
            Some(res.lang.t(difficulty).to_owned()),
        ),
        (
            MenuItem::Language,
            "menu_language",
            Some(res.lang.t(language).to_owned()),
        ),
        (
            MenuItem::MasterVolume,
            "menu_master_volume",
            Some(format!("{:.0}%", settings.master_volume * 100.0)),
        ),
        (
            MenuItem::MusicVolume,
            "menu_music_volume",
            Some(format!("{:.0}%", settings.music_volume * 100.0)),
        ),
        (
            MenuItem::SfxVolume,
            "menu_sfx_volume",
            Some(format!("{:.0}%", settings.sfx_volume * 100.0)),
        ),
        (
            MenuItem::AudioMute,
            "menu_audio_mute",
            Some(res.lang.t(audio_status).to_owned()),
        ),
    ];

    let mut click = None;
    for (index, (item, key, value)) in rows.into_iter().enumerate() {
        let bounds = Rect::new(
            row_x,
            start_y + index as f32 * row_step,
            row_width,
            row_step * 0.88,
        );
        let selected = game.menu_selection == item;
        let label = res.lang.t(key);
        let text = value.map_or_else(|| label.to_owned(), |value| format!("{label}  {value}"));

        if matches!(
            item,
            MenuItem::MasterVolume | MenuItem::MusicVolume | MenuItem::SfxVolume
        ) {
            let side_width = (row_width * 0.16).min(48.0);
            let left = Rect::new(bounds.x, bounds.y, side_width, bounds.h);
            let center = Rect::new(
                bounds.x + side_width + 4.0,
                bounds.y,
                bounds.w - (side_width + 4.0) * 2.0,
                bounds.h,
            );
            let right = Rect::new(bounds.right() - side_width, bounds.y, side_width, bounds.h);

            if menu_button("-", left, 22, false, res) {
                click = Some(MenuClick::DecreaseVolume(item));
            }
            if menu_button(&text, center, 18, selected, res) {
                click = Some(MenuClick::Select(item));
            }
            if menu_button("+", right, 22, false, res) {
                click = Some(MenuClick::IncreaseVolume(item));
            }
        } else {
            let size = if item == MenuItem::Start { 26 } else { 18 };
            if menu_button(&text, bounds, size, selected, res) {
                click = Some(MenuClick::Activate(item));
            }
        }
    }

    draw_text_centered(
        res.lang.t("menu_instructions"),
        start_y + 7.0 * row_step + 22.0 - screen_h / 2.0,
        14,
        GRAY,
        res,
    );
    draw_text_with_font(
        concat!("v", env!("CARGO_PKG_VERSION")),
        16.0,
        screen_h - 14.0,
        14.0,
        LIGHTGRAY,
        res,
    );
    click
}

pub enum ShipSelectionClick {
    Select(ShipKind),
    Confirm,
    Back,
}

pub fn render_ship_selection(selected: ShipKind, res: &Resources) -> Option<ShipSelectionClick> {
    let center = vec2(screen_width() / 2.0, screen_height() / 2.0);
    let card_width = (screen_width() * 0.43).min(260.0);
    let card_height = (screen_height() * 0.4).min(218.0);
    let gap = 18.0;
    let has_delta = res.ship_model(ShipKind::Delta).is_some();
    let x = if has_delta {
        center.x - card_width - gap / 2.0
    } else {
        center.x - card_width / 2.0
    };
    let y = center.y - card_height / 2.0;
    draw_text_centered(
        res.lang.t("ship_selection"),
        -card_height / 2.0 - 40.0,
        30,
        ORANGE,
        res,
    );

    let mut click = None;
    for (kind, label, shortcut, x) in [
        (ShipKind::Alpha, "ship_alpha", "←", x),
        (ShipKind::Delta, "ship_delta", "→", x + card_width + gap),
    ] {
        if kind == ShipKind::Delta && !has_delta {
            continue;
        }
        let card = Rect::new(x, y, card_width, card_height);
        if draw_menu_button(card, kind == selected) {
            click = Some(ShipSelectionClick::Select(kind));
        }
        if has_delta {
            let key_bounds = Rect::new(x + 8.0, y + 8.0, 30.0, 24.0);
            draw_menu_label(shortcut, key_bounds, 16, LIGHTGRAY, res);
        }
        draw_ship_hull(
            kind,
            vec2(x + card_width / 2.0, y + card_height * 0.43),
            (card_height * 0.73).min(card_width * 0.86),
            0.0,
            0.0,
            WHITE,
            res,
        );
        let label_bounds = Rect::new(x, y + card_height * 0.76, card_width, card_height * 0.2);
        draw_menu_label(res.lang.t(label), label_bounds, 20, WHITE, res);
    }
    if centered_menu_button(res.lang.t("ship_confirm"), card_height / 2.0 + 32.0, res) {
        return Some(ShipSelectionClick::Confirm);
    }
    if centered_menu_button(res.lang.t("ship_back"), card_height / 2.0 + 82.0, res) {
        return Some(ShipSelectionClick::Back);
    }
    click
}

fn objective_label(objective: MissionObjective) -> &'static str {
    match objective {
        MissionObjective::DefeatEnemies(_) => "goal_enemies",
        MissionObjective::DestroyAsteroids(_) => "goal_asteroids",
        MissionObjective::DefeatInterceptors(_) => "goal_interceptors",
        MissionObjective::DefeatHeavyShips(_) => "goal_heavy_ships",
        MissionObjective::CollectScrap(_) => "goal_scrap",
        MissionObjective::Survive(_) => "goal_survive",
        MissionObjective::DefeatBoss => "obj_destroy_boss",
    }
}

pub fn render_briefing(mission: &Mission, res: &Resources) -> bool {
    draw_text_centered(
        &format!("{} {}", res.lang.t("mission"), mission.level_id),
        -100.0,
        32,
        ORANGE,
        res,
    );
    draw_text_centered(&mission.title, -50.0, 48, WHITE, res);
    draw_text_centered(res.lang.t("objectives"), 20.0, 24, GRAY, res);

    for (row, &objective) in mission.objectives.iter().enumerate() {
        let label = res.lang.t(objective_label(objective));
        let text = if objective == MissionObjective::DefeatBoss {
            label.to_string()
        } else {
            format!("{label}  {}", objective.target())
        };
        draw_text_centered(&text, 60.0 + row as f32 * 32.0, 24, WHITE, res);
    }

    centered_menu_button(res.lang.t("press_space"), 190.0, res)
}

pub fn render_mission_success(mission: &Mission, res: &Resources) -> bool {
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
    centered_menu_button(res.lang.t("next_mission"), 100.0, res)
}

pub fn render_upgrade_bay(game: &Game, res: &Resources) -> Option<usize> {
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

    let start_y = screen_height() * 0.5 - 130.0;
    let line_height = 52.0;
    let row_width = (screen_width() * 0.94).min(850.0);
    let row_x = (screen_width() - row_width) / 2.0;
    let mut clicked = None;

    for (idx, id) in UPGRADABLE_IDS.iter().copied().enumerate() {
        let def = upgrade_definition(id);
        let level = game.upgrade_level(id);
        let max_level = def.max_level;
        let selected = game.upgrade_selection == idx;

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
            "{}  [{} {}/{}]  {}",
            res.lang.t(def.name_key),
            res.lang.t("upgrade_level"),
            level,
            max_level,
            status_text
        );

        let y = start_y + line_height * idx as f32;
        let bounds = Rect::new(row_x, y, row_width, 46.0);
        if draw_menu_button(bounds, selected) {
            clicked = Some(idx);
        }
        draw_menu_label(
            &row_text,
            Rect::new(row_x, y, row_width, 26.0),
            20,
            WHITE,
            res,
        );
        draw_menu_label(
            res.lang.t(def.desc_key),
            Rect::new(row_x, y + 25.0, row_width, 18.0),
            14,
            LIGHTGRAY,
            res,
        );
    }

    let continue_idx = UPGRADABLE_IDS.len();
    let bounds = Rect::new(
        row_x,
        start_y + line_height * continue_idx as f32,
        row_width,
        40.0,
    );
    if menu_button(
        res.lang.t("upgrade_continue"),
        bounds,
        24,
        game.upgrade_selection == continue_idx,
        res,
    ) {
        clicked = Some(continue_idx);
    }

    draw_text_centered(res.lang.t("upgrade_controls"), 205.0, 16, GRAY, res);
    clicked
}

pub fn render_game_over(score: u32, high_score: u32, res: &Resources) -> bool {
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
    centered_menu_button(res.lang.t("return_to_menu"), 115.0, res)
}

pub fn render_pause(res: &Resources) -> (bool, bool) {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.0, 0.0, 0.0, 0.7),
    );

    draw_text_centered(res.lang.t("paused"), -40.0, 48, YELLOW, res);
    let resume = centered_menu_button(res.lang.t("press_enter_resume"), 20.0, res);
    let main_menu = centered_menu_button(res.lang.t("press_esc"), 70.0, res);
    (resume, main_menu)
}
