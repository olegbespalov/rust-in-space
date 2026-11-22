use crate::components::{
    Asteroid, AudioSettings, Boss, BossAttackKind, BossPhase, BulletStyle, DevMenuItem, Difficulty,
    EnemyShip, EnemyType, Engine, Explosion, LootItem, LootType, MenuItem, Mission,
    MissionObjective, ShieldKind, Ship, ShipKind, UpgradeId, DEFAULT_PLAYER_NAME,
};
use crate::game::constants::{
    BOSS_SIZE, PLAYER_SHIP_DRAW_SIZE, SHIELD_DRAW_SCALE, SHIP_VISUAL_SCALE,
};
use crate::game::{upgrade_definition, Game, Pilot, UpgradeSelection, MAX_PILOTS, UPGRADABLE_IDS};
use crate::model_sprite::ModelSprite;
use crate::resources::Resources;
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use std::borrow::Cow;

pub fn draw_text_centered(text: &str, y_offset: f32, size: u16, color: Color, res: &Resources) {
    let font = res.text_font(text);
    let params = TextParams {
        font,
        font_size: size,
        color,
        ..Default::default()
    };

    let dims = measure_text(text, font, size, 1.0);
    draw_text_ex(
        text,
        screen_width() / 2.0 - dims.width / 2.0,
        screen_height() / 2.0 - dims.height / 2.0 + y_offset,
        params,
    );
}

pub fn draw_text_with_font(text: &str, x: f32, y: f32, size: f32, color: Color, res: &Resources) {
    let font = res.text_font(text);
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
    draw_background_in(texture, vec2(screen_width(), screen_height()));
}

pub fn draw_background_in(texture: &Texture2D, bounds: Vec2) {
    let screen_w = bounds.x;
    let screen_h = bounds.y;
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

    let ship_size = PLAYER_SHIP_DRAW_SIZE;
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
    let size = hull_size * SHIELD_DRAW_SCALE;
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

pub fn draw_ship_hull(
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

fn draw_launch_engine(
    position: Vec2,
    hull_rotation: f32,
    hull_size: f32,
    thrust: f32,
    alpha: f32,
    texture: &Texture2D,
) {
    if thrust <= 0.05 {
        return;
    }
    let scale = hull_size / 72.0;
    let engine_direction = hull_rotation - std::f32::consts::FRAC_PI_2;
    let direction = vec2(engine_direction.cos(), engine_direction.sin());
    let width = 22.0 * scale * thrust;
    let flicker = gen_range(-3.0, 3.0) * scale * thrust;
    let height = 52.0 * scale * thrust + flicker;
    let flame_pos = position - direction * (hull_size * 0.58);
    draw_texture_ex(
        texture,
        flame_pos.x - width / 2.0,
        flame_pos.y - height / 2.0,
        Color { a: alpha, ..WHITE },
        DrawTextureParams {
            dest_size: Some(vec2(width, height)),
            rotation: hull_rotation,
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

fn health_bar_left(center_x: f32, width: f32, field_width: f32) -> f32 {
    (center_x - width / 2.0).clamp(0.0, (field_width - width).max(0.0))
}

fn draw_enemy(enemy: &EnemyShip, field_width: f32, res: &Resources) {
    let sprite = match enemy.enemy_type {
        EnemyType::Regular => &res.enemy_small,
        EnemyType::Kamikaze => &res.enemy_kamikaze,
        EnemyType::Interceptor => &res.enemy_interceptor,
        EnemyType::Heavy => &res.enemy_heavy,
        EnemyType::Guardian => {
            if enemy.health <= enemy.max_health / 2.0 {
                &res.enemy_guardian_damaged
            } else {
                &res.enemy_guardian
            }
        }
    };
    let size = enemy.enemy_type.draw_size();

    let bar_left = health_bar_left(enemy.pos.x, size, field_width);
    let bar_top = health_bar_top(
        enemy.pos.y,
        size / 2.0,
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

const BOSS_HP_BAR_HEIGHT: f32 = 10.0;
const BOSS_HP_BAR_GAP: f32 = 8.0;

fn draw_boss(boss: &Boss, field_width: f32, res: &Resources) {
    let size = BOSS_SIZE;
    let bar_width = size;
    let bar_left = health_bar_left(boss.pos.x, bar_width, field_width);
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

    if !boss.power_ups.is_invisible() {
        for phase in 1..BossPhase::COUNT {
            let x = bar_left + bar_width * f32::from(phase) / f32::from(BossPhase::COUNT);
            draw_line(x, bar_top, x, bar_top + BOSS_HP_BAR_HEIGHT, 2.0, BLACK);
        }
    }
    draw_texture_ex(
        &res.boss_dreadnought,
        boss.pos.x - size / 2.0,
        boss.pos.y - size / 2.0,
        tint,
        DrawTextureParams {
            dest_size: Some(Vec2::splat(size)),
            rotation: boss.rotation + std::f32::consts::FRAC_PI_2,
            ..Default::default()
        },
    );
    if boss.phase >= BossPhase::MissileSalvos {
        let launcher_light = Color::new(SKYBLUE.r, SKYBLUE.g, SKYBLUE.b, tint.a * 0.85);
        for (launcher, _) in boss.attack_paths(BossAttackKind::Missiles, 0) {
            draw_circle(launcher.x, launcher.y, 3.5, launcher_light);
        }
    }
    if boss.phase == BossPhase::Escalated {
        let fwd = Vec2::from_angle(boss.rotation);
        let core = boss.pos - fwd * (size * 0.19);
        draw_circle(
            core.x,
            core.y,
            20.0,
            Color::new(1.0, 0.14, 0.42, tint.a * 0.42),
        );
        draw_circle_lines(
            core.x,
            core.y,
            19.0,
            2.0,
            Color::new(1.0, 0.49, 0.31, tint.a),
        );
    }
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

fn draw_boss_attack(boss: &Boss) {
    let Some(attack) = boss.pending_attack else {
        return;
    };
    let charge = (1.0 - attack.remaining / attack.kind.warning_duration()).clamp(0.0, 1.0);
    let (color, length) = match attack.kind {
        BossAttackKind::Cannons => (ORANGE, 570.0),
        BossAttackKind::Missiles => (SKYBLUE, 95.0),
        BossAttackKind::Rings => (VIOLET, 140.0),
    };
    let mut line_color = color;
    line_color.a = 0.35 + charge * 0.45;
    for (index, (from, dir)) in boss.attack_paths(attack.kind, 0).enumerate() {
        let to = from + dir * length;
        draw_line(from.x, from.y, to.x, to.y, 2.0 + 2.0 * charge, line_color);
        if attack.kind != BossAttackKind::Cannons || index.is_multiple_of(3) {
            draw_circle(from.x, from.y, 4.0 + 5.0 * charge, color);
        }
    }
    if attack.kind == BossAttackKind::Rings {
        draw_circle_lines(
            boss.pos.x,
            boss.pos.y,
            BOSS_SIZE * 0.48,
            2.0 + 4.0 * charge,
            line_color,
        );
    }
}

fn draw_boss_missile(pos: Vec2, vel: Vec2, radius: f32) {
    let forward = vel.normalize_or_zero() * radius;
    let wing = forward.perp() * 0.48;
    let tip = pos + forward * 1.12;
    let tail = pos - forward * 0.66;
    let exhaust = pos - forward * 1.65;
    draw_triangle(tail + wing * 0.64, exhaust, tail - wing * 0.64, ORANGE);
    draw_triangle(tip, tail + wing, tail - wing, SKYBLUE);
    draw_triangle(tip, tail + wing * 0.28, tail - wing * 0.28, WHITE);
}

fn draw_enemy_health_bar(x: f32, y: f32, width: f32, height: f32, ratio: f32) {
    draw_rectangle(x, y, width, height, ENEMY_HP_BAR_BG_COLOR);
    let ratio = ratio.clamp(0.0, 1.0);
    if ratio > 0.0 {
        let hp_color = Color::new(1.0 - ratio, ratio, 0.0, 1.0);
        draw_rectangle(x, y, width * ratio, height, hp_color);
    }
}

fn draw_loot(item: &LootItem, bounds: Vec2, res: &Resources) {
    let size = item.radius * 4.5;
    let sprite = match item.item_type {
        LootType::Scrap(_) => &res.loot_scrap,
        LootType::MediumScrap(_) => &res.loot_medium_scrap,
        LootType::LargeScrap(_) => &res.loot_large_scrap,
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
    let x = if right + width + TIMER_GAP <= bounds.x {
        right.max(TIMER_GAP)
    } else {
        item.pos.x - size * 0.5 - TIMER_GAP - width
    };
    let font_size = f32::from(TIMER_FONT_SIZE);
    let y = (item.pos.y + font_size / 3.0).clamp(font_size, (bounds.y - 1.0).max(font_size));
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

fn draw_enemy_direction_hint(from_left: bool, spawn_y: f32, t: f32, bounds: Vec2) {
    let blink = 0.4 + 0.6 * (t * 2.5).sin().mul_add(0.5, 0.5);
    let color = Color::new(0.2, 1.0, 0.4, blink);

    let w = bounds.x;
    let h = bounds.y;
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

#[derive(Clone, Copy)]
pub struct LanView<'a> {
    pub slot: usize,
    pub focus: usize,
    pub area: Rect,
    pub names: [&'a str; MAX_PILOTS],
    pub local_ship: Option<&'a Ship>,
}

impl<'a> LanView<'a> {
    fn pilot_color(self, slot: usize) -> Color {
        if slot == self.slot {
            SKYBLUE
        } else if slot == self.focus {
            GREEN
        } else {
            GOLD
        }
    }

    fn pilot_ship(self, slot: usize) -> Option<&'a Ship> {
        self.local_ship.filter(|_| self.slot == slot)
    }
}

pub fn render_game(game: &Game, resources: &Resources) {
    render_battle_world(game, None, resources);
    render_battle_hud(game, None, resources);
}

pub fn render_battle_world(game: &Game, lan: Option<LanView<'_>>, resources: &Resources) {
    let bounds = if lan.is_some() {
        game.bounds
    } else {
        vec2(screen_width(), screen_height())
    };
    let visible = |pos, padding| {
        lan.is_none_or(|view| {
            let area = view.area;
            Rect::new(
                area.x - padding,
                area.y - padding,
                area.w + padding * 2.0,
                area.h + padding * 2.0,
            )
            .contains(pos)
        })
    };
    for item in game
        .loot_items
        .iter()
        .filter(|item| visible(item.pos, item.radius * 2.25 + 44.0))
    {
        draw_loot(item, bounds, resources);
    }

    for b in game
        .bullets
        .iter()
        .filter(|b| visible(b.pos, b.radius * 1.5))
    {
        let rotation = b.vel.y.atan2(b.vel.x) + std::f32::consts::FRAC_PI_2;
        let size = b.radius * 2.0;
        match b.style {
            BulletStyle::Enemy => {
                resources.enemy_bullet.draw(b.pos, size, rotation);
                continue;
            }
            BulletStyle::Boss => {
                draw_circle(b.pos.x, b.pos.y, b.radius, ORANGE);
                draw_circle(b.pos.x, b.pos.y, b.radius * 0.55, YELLOW);
                continue;
            }
            BulletStyle::BossMissile => {
                draw_boss_missile(b.pos, b.vel, b.radius);
                continue;
            }
            BulletStyle::Player => {}
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

    for a in game
        .asteroids
        .iter()
        .filter(|a| visible(a.pos, a.radius * 1.5))
    {
        draw_asteroid(a, resources);
    }

    for e in game.enemy_ships.iter().filter(|e| visible(e.pos, 100.0)) {
        draw_enemy(e, bounds.x, resources);
    }

    if let Some(boss) = game
        .boss
        .as_ref()
        .filter(|boss| visible(boss.pos, BOSS_SIZE))
    {
        draw_boss(boss, bounds.x, resources);
    }
    if let Some(boss) = &game.boss {
        draw_boss_attack(boss);
    }

    for ex in game.explosions.iter().filter(|ex| {
        let sprite = &resources.explosion;
        let size = (sprite.width() / ex.max_frames as f32).max(sprite.height());
        visible(ex.pos, size * ex.scale / 2.0)
    }) {
        draw_explosion(ex, resources);
    }

    for (slot, pilot) in game.pilots.iter().enumerate() {
        if let Some(pilot) = pilot.as_ref().filter(|pilot| pilot.is_alive()) {
            let ship = lan
                .and_then(|view| view.pilot_ship(slot))
                .unwrap_or(&pilot.ship);
            let pos = ship.pos;
            if !visible(pos, 100.0) {
                continue;
            }
            draw_ship(ship, resources);
            if let Some(view) = lan.filter(|view| view.area.contains(pos)) {
                let label = format!("{} · {}", slot + 1, view.names[slot]);
                let x = (pos.x - 110.0).clamp(view.area.left(), view.area.right() - 220.0);
                let y = if pos.y + 55.0 <= view.area.bottom() {
                    pos.y + 30.0
                } else {
                    pos.y - 55.0
                };
                let bounds = Rect::new(x, y, 220.0, 25.0);
                let shadow = Rect::new(bounds.x + 1.0, bounds.y + 1.0, bounds.w, bounds.h);
                draw_menu_label(&label, shadow, 16, BLACK, resources);
                draw_menu_label(&label, bounds, 16, view.pilot_color(slot), resources);
            }
        }
    }
}

pub fn render_battle_hud(game: &Game, lan: Option<LanView<'_>>, resources: &Resources) {
    let bounds = lan.map_or_else(
        || vec2(screen_width(), screen_height()),
        |view| vec2(view.area.w, view.area.h),
    );
    let local_slot = lan.map_or(0, |view| view.slot);
    let Some(pilot) = &game.pilots[local_slot] else {
        return;
    };
    let ship = &pilot.ship;

    let mut status_text = format!(
        "{} {}  {} {:.0}/{:.0}",
        resources.lang.t("score"),
        game.score,
        resources.lang.t("hp"),
        ship.health.max(0.0),
        ship.max_health
    );
    if lan.is_some() {
        status_text = format!(
            "{} {}  ·  {status_text}",
            resources.lang.t("mission"),
            game.current_mission.level_id
        );
    }
    if ship.has_shield() {
        status_text.push_str(&format!(
            "  {} {:.0}/{:.0}",
            resources.lang.t("shield"),
            ship.shield_hp,
            ship.shield_max_hp
        ));
    }
    draw_text_with_font(&status_text, 20.0, 30.0, 24.0, WHITE, resources);
    if let Some(view) = lan.filter(|_| !pilot.is_alive()) {
        let mut label = format!(
            "{} · {}",
            resources.lang.t("lan_spectating"),
            view.names[view.focus],
        );
        if game
            .pilots
            .iter()
            .flatten()
            .filter(|pilot| pilot.is_alive())
            .count()
            > 1
        {
            label.push_str("   ");
            label.push_str(&resources.lang.t("lan_switch_pilot"));
        }
        draw_text_with_font(&label, 20.0, 60.0, 20.0, GOLD, resources);
    } else if ship.is_invisible() {
        let cloak_text = format!(
            "{} {:.0}",
            resources.lang.t("invisibility"),
            ship.invisibility_timer.ceil()
        );
        draw_text_with_font(&cloak_text, 20.0, 60.0, 24.0, SKYBLUE, resources);
    }

    let objectives = &game.current_mission.objectives;
    for (row, &objective) in objectives.iter().enumerate() {
        let complete = game.mission_progress.meets(objective);
        let status = match objective {
            MissionObjective::DefeatBoss => match &game.boss {
                Some(boss) => {
                    format!(
                        "{} {:.0}/{:.0}  {} {}/{}",
                        resources.lang.t("boss_hp"),
                        boss.health.ceil(),
                        boss.max_health,
                        resources.lang.t("boss_phase"),
                        boss.phase.number(),
                        BossPhase::COUNT,
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
            bounds.y - 30.0 - (objectives.len() - 1 - row) as f32 * 26.0,
            24.0,
            if complete { GREEN } else { WHITE },
            resources,
        );
    }

    let inventory = format!(
        "{} {} {} | {} {}",
        resources.lang.t("resources"),
        resources.lang.t("rust"),
        ship.scrap,
        resources.lang.t("gold"),
        ship.rare_metal
    );
    draw_text_with_font(
        &inventory,
        20.0,
        bounds.y - 34.0 - objectives.len() as f32 * 26.0,
        20.0,
        GRAY,
        resources,
    );

    let origin_y = lan.map_or(0.0, |view| view.area.y);
    if let Some(boss) = &game.boss {
        if let Some(wave) = boss.pending_summon {
            for (from_left, y, _) in wave.arrivals(boss.phase) {
                draw_enemy_direction_hint(from_left, y - origin_y, -wave.remaining, bounds);
            }
        }
    } else if game.difficulty == Difficulty::Nebula {
        if let Some((from_left, spawn_y, _)) = game.pending_enemy_hint {
            draw_enemy_direction_hint(from_left, spawn_y - origin_y, get_time() as f32, bounds);
        }
    }

    if let Some(view) = lan {
        draw_lan_minimap(game, view, resources);
    }
}

fn draw_lan_minimap(game: &Game, view: LanView<'_>, res: &Resources) {
    const WIDTH: f32 = 168.0;
    let scale = WIDTH / game.bounds.x;
    let height = game.bounds.y * scale;
    let origin = vec2(view.area.w - WIDTH - 68.0, 20.0);
    let area = view.area;
    draw_rectangle(
        origin.x,
        origin.y,
        WIDTH,
        height,
        Color::new(0.0, 0.0, 0.0, 0.65),
    );
    draw_rectangle_lines(origin.x, origin.y, WIDTH, height, 1.0, GRAY);
    draw_rectangle_lines(
        origin.x + area.x * scale,
        origin.y + area.y * scale,
        area.w * scale,
        area.h * scale,
        1.0,
        SKYBLUE,
    );
    let dot = |pos: Vec2, radius, color| {
        let pos = origin + pos.clamp(Vec2::ZERO, game.bounds) * scale;
        draw_circle(pos.x, pos.y, radius, color);
    };
    for asteroid in &game.asteroids {
        dot(asteroid.pos, 1.5, GRAY);
    }
    for enemy in &game.enemy_ships {
        if !enemy.power_ups.is_invisible() {
            dot(enemy.pos, 2.0, RED);
        }
    }
    if let Some(boss) = game
        .boss
        .as_ref()
        .filter(|boss| !boss.power_ups.is_invisible())
    {
        dot(boss.pos, 3.5, RED);
    }
    let mut row_y = origin.y + height + 4.0;
    for (slot, pilot) in game.pilots.iter().enumerate() {
        if let Some(pilot) = pilot {
            let color = if pilot.is_alive() {
                let color = view.pilot_color(slot);
                let ship = view.pilot_ship(slot).unwrap_or(&pilot.ship);
                let pos = origin + ship.pos.clamp(Vec2::ZERO, game.bounds) * scale;
                draw_circle(pos.x, pos.y, 8.0, color);
                draw_menu_label(
                    &(slot + 1).to_string(),
                    Rect::new(pos.x - 10.0, pos.y - 12.0, 20.0, 24.0),
                    12,
                    BLACK,
                    res,
                );
                color
            } else {
                LIGHTGRAY
            };
            let row = Rect::new(origin.x, row_y, WIDTH, 23.0);
            draw_rectangle(row.x, row.y, row.w, row.h, Color::new(0.0, 0.0, 0.0, 0.65));
            let mut name_bounds = row;
            if slot != view.slot {
                const HP_WIDTH: f32 = 66.0;
                name_bounds.w -= HP_WIDTH;
                let ship = &pilot.ship;
                draw_menu_label(
                    &format!("{:.0}/{:.0}", ship.health.max(0.0), ship.max_health),
                    Rect::new(name_bounds.right(), row.y, HP_WIDTH, row.h),
                    15,
                    color,
                    res,
                );
            }
            draw_menu_label(
                &format!("{} · {}", slot + 1, view.names[slot]),
                name_bounds,
                15,
                color,
                res,
            );
            row_y += row.h;
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

pub fn draw_menu_label(text: &str, bounds: Rect, size: u16, color: Color, res: &Resources) {
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
    let font = res.text_font(text);
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

pub fn menu_button(text: &str, bounds: Rect, size: u16, selected: bool, res: &Resources) -> bool {
    let clicked = draw_menu_button(bounds, selected);
    draw_menu_label_with_scale(text, bounds, size, WHITE, 0.84, res);
    clicked
}

pub fn centered_menu_button(text: &str, y_offset: f32, res: &Resources) -> bool {
    centered_menu_button_with_height(text, y_offset, MENU_BUTTON_HEIGHT, res)
}

pub const MENU_BUTTON_HEIGHT: f32 = 52.0;

pub fn centered_menu_button_with_height(
    text: &str,
    y_offset: f32,
    height: f32,
    res: &Resources,
) -> bool {
    let width =
        (measure_text(text, res.text_font(text), 24, 1.0).width + 64.0).min(screen_width() * 0.94);
    let bounds = Rect::new(
        (screen_width() - width) / 2.0,
        screen_height() / 2.0 + y_offset - height / 2.0,
        width,
        height,
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

pub fn difficulty_key(difficulty: Difficulty) -> &'static str {
    match difficulty {
        Difficulty::Nebula => "diff_nebula",
        Difficulty::Supernova => "diff_supernova",
        Difficulty::BlackHole => "diff_blackhole",
    }
}

pub fn render_menu(
    selection: MenuItem,
    difficulty: Difficulty,
    player_name: &str,
    high_score: u32,
    settings: &AudioSettings,
    res: &Resources,
) -> Option<MenuClick> {
    let screen_w = screen_width();
    let screen_h = screen_height();
    let pulse = 1.0 + (get_time() * 2.0).sin() as f32 * 0.05;
    let aspect_ratio = res.logo.height() / res.logo.width();
    let logo_w = (screen_w * 0.3).min(screen_h * 0.23 / aspect_ratio) * pulse;
    let logo_h = logo_w * aspect_ratio;
    draw_texture_ex(
        &res.logo,
        (screen_w - logo_w) / 2.0,
        screen_h * 0.15 - logo_h / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(logo_w, logo_h)),
            ..Default::default()
        },
    );

    let start_y = screen_h * 0.37;
    let row_step = ((screen_h * 0.63 - 64.0) / 9.0).clamp(1.0, 48.0);
    let row_height = (row_step * 0.8).min(42.0);
    let row_width = (screen_w * 0.94).min(420.0);
    let row_x = (screen_w - row_width) / 2.0;
    draw_menu_label(
        &pilot_label("player_welcome", player_name, res),
        Rect::new(0.0, screen_h * 0.26, screen_w, screen_h * 0.06),
        20,
        WHITE,
        res,
    );
    draw_menu_label(
        &format!("{} {}", res.lang.t("high_score"), high_score),
        Rect::new(0.0, screen_h * 0.32, screen_w, screen_h * 0.06),
        18,
        YELLOW,
        res,
    );

    let difficulty = difficulty_key(difficulty);
    let audio_status = if settings.audio_muted {
        "audio_off"
    } else {
        "audio_on"
    };

    let rows = [
        (MenuItem::Start, "menu_start", None),
        (MenuItem::Lan, "lan_menu", None),
        (
            MenuItem::Difficulty,
            "menu_difficulty",
            Some(res.lang.t(difficulty).into_owned()),
        ),
        (
            MenuItem::Language,
            "menu_language",
            Some(res.lang.t("language_name").into_owned()),
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
            Some(res.lang.t(audio_status).into_owned()),
        ),
        (MenuItem::EnemyCodex, "enemy_codex", None),
    ];

    let row_count = rows.len();
    let mut click = None;
    for (index, (item, key, value)) in rows.into_iter().enumerate() {
        let bounds = Rect::new(
            row_x,
            start_y + index as f32 * row_step,
            row_width,
            row_height,
        );
        let selected = selection == item;
        let label = res.lang.t(key);
        let text = match value {
            Some(value) => Cow::Owned(format!("{label}  {value}")),
            None => label,
        };

        if matches!(
            item,
            MenuItem::MasterVolume | MenuItem::MusicVolume | MenuItem::SfxVolume
        ) {
            let side_width = (row_width * 0.16).min(48.0);
            let left = Rect::new(bounds.x, bounds.y, side_width, bounds.h);
            let center = Rect::new(
                bounds.x + side_width + 8.0,
                bounds.y,
                bounds.w - (side_width + 8.0) * 2.0,
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
        &res.lang.t("menu_instructions"),
        start_y + row_count as f32 * row_step + 22.0 - screen_h / 2.0,
        14,
        GRAY,
        res,
    );
    draw_text_with_font(crate::VERSION, 16.0, screen_h - 14.0, 14.0, LIGHTGRAY, res);
    click
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn render_update_footer(update: &crate::update::Update, res: &Resources) -> bool {
    use crate::update::Update;
    use fluent_bundle::FluentArgs;

    let (key, tag) = match update {
        Update::Ready => ("update_check", None),
        Update::Checking(_) => ("update_checking", None),
        Update::Checked { failed: false, .. } => ("update_current", None),
        Update::Checked { failed: true, .. } => ("update_check_failed", None),
        Update::Available(release) if release.can_install() => {
            ("update_install", Some(&release.tag))
        }
        Update::Available(release) => ("update_github", Some(&release.tag)),
        Update::Installing(_) => ("update_installing", None),
        Update::Failed(_) => ("update_failed", None),
        Update::Installed => ("update_installed", None),
        Update::Hidden => return false,
    };
    let mut args = FluentArgs::new();
    if let Some(tag) = tag {
        args.set("version", tag.as_str());
    }
    let text = res.lang.format(key, Some(&args));
    let version_width = measure_text(crate::VERSION, res.text_font(crate::VERSION), 14, 1.0).width;
    let x = 32.0 + version_width;
    let width = measure_text(&text, res.text_font(&text), 14, 1.0).width + 32.0;
    let bounds = Rect::new(
        x,
        screen_height() - 36.0,
        width.min(screen_width() - x - 12.0),
        28.0,
    );
    if bounds.w <= 0.0 {
        return false;
    }
    if matches!(
        update,
        Update::Checking(_) | Update::Checked { .. } | Update::Installing(_)
    ) {
        draw_menu_label(&text, bounds, 14, LIGHTGRAY, res);
        false
    } else {
        menu_button(&text, bounds, 14, false, res)
    }
}

pub fn render_player_name(draft: &str, res: &Resources) -> bool {
    draw_menu_label(
        &res.lang.t("player_name_prompt"),
        Rect::new(0.0, screen_height() / 2.0 - 120.0, screen_width(), 58.0),
        30,
        ORANGE,
        res,
    );
    let width = (screen_width() * 0.9).min(520.0);
    let field = Rect::new(
        (screen_width() - width) / 2.0,
        screen_height() / 2.0 - 23.0,
        width,
        46.0,
    );
    draw_menu_button(field, true);
    let (text, color) = if draft.is_empty() {
        (DEFAULT_PLAYER_NAME, LIGHTGRAY)
    } else {
        (draft, WHITE)
    };
    draw_menu_label(&format!("{text}_"), field, 20, color, res);
    centered_menu_button(&res.lang.t("ship_confirm"), 77.0, res)
}

pub enum ShipSelectionClick {
    Select(ShipKind),
    Confirm,
    Back,
}

pub const SHIP_CARD_HEIGHT: f32 = 218.0;

pub enum DevMenuClick {
    Level,
    Ship(ShipKind),
    Start,
    Shop,
}

pub fn render_dev_menu(
    selection: DevMenuItem,
    level: u32,
    ship: ShipKind,
    res: &Resources,
) -> Option<DevMenuClick> {
    let width = screen_width();
    let height = screen_height();
    let band = |y, h| Rect::new(0.0, y * height, width, h * height);
    let button = |center: f32| {
        let w = (width * 0.9).min(650.0);
        let h = (height * 0.085).min(MENU_BUTTON_HEIGHT);
        Rect::new((width - w) / 2.0, height * center - h / 2.0, w, h)
    };

    draw_menu_label(&res.lang.t("dev_title"), band(0.04, 0.12), 36, ORANGE, res);
    let level_clicked = menu_button(
        &format!("←  {} {level}  →", res.lang.t("level_cleared_prefix")),
        button(0.22),
        24,
        selection == DevMenuItem::Level,
        res,
    );
    let ship_color = if selection == DevMenuItem::Ship {
        YELLOW
    } else {
        WHITE
    };
    draw_menu_label(
        &res.lang.t("ship_selection"),
        band(0.27, 0.07),
        22,
        ship_color,
        res,
    );
    let ship_clicked = render_ship_cards(
        ship,
        height * 0.35,
        (height * 0.32).min(SHIP_CARD_HEIGHT),
        res,
    );
    let start_clicked = menu_button(
        &res.lang.t("dev_start_level"),
        button(0.73),
        24,
        selection == DevMenuItem::Start,
        res,
    );
    let shop_clicked = menu_button(
        &res.lang.t("dev_open_shop"),
        button(0.83),
        24,
        selection == DevMenuItem::Shop,
        res,
    );
    draw_menu_label(
        &res.lang.t("dev_controls"),
        band(0.89, 0.07),
        16,
        LIGHTGRAY,
        res,
    );

    level_clicked
        .then_some(DevMenuClick::Level)
        .or(ship_clicked.map(DevMenuClick::Ship))
        .or(start_clicked.then_some(DevMenuClick::Start))
        .or(shop_clicked.then_some(DevMenuClick::Shop))
}

pub fn render_ship_selection(selected: ShipKind, res: &Resources) -> Option<ShipSelectionClick> {
    let screen_h = screen_height();
    let card_height = (screen_h * 0.4).min(SHIP_CARD_HEIGHT);
    let scale = (screen_h / 500.0).min(1.0);
    let button_height = MENU_BUTTON_HEIGHT * scale;
    let gap = 18.0 * scale;
    draw_text_centered(
        &res.lang.t("ship_selection"),
        -card_height / 2.0 - 40.0 * scale,
        30,
        ORANGE,
        res,
    );
    let click = render_ship_cards(selected, (screen_h - card_height) / 2.0, card_height, res);
    let confirm_y = card_height / 2.0 + gap + button_height / 2.0;
    if centered_menu_button_with_height(&res.lang.t("ship_confirm"), confirm_y, button_height, res)
    {
        return Some(ShipSelectionClick::Confirm);
    }
    let back_y = confirm_y + button_height + gap;
    if centered_menu_button_with_height(&res.lang.t("ship_back"), back_y, button_height, res) {
        return Some(ShipSelectionClick::Back);
    }
    click.map(ShipSelectionClick::Select)
}

pub fn keyboard_ship_selection(selected: ShipKind, res: &Resources) -> ShipKind {
    let direction = match (
        is_key_pressed(KeyCode::Left),
        is_key_pressed(KeyCode::Right),
    ) {
        (true, false) => -1,
        (false, true) => 1,
        _ => return selected,
    };
    let ships: Vec<_> = res.selectable_ships().collect();
    let Some(index) = ships.iter().position(|&kind| kind == selected) else {
        return ShipKind::Alpha;
    };
    let next = (index as isize + direction).rem_euclid(ships.len() as isize);
    ships[next as usize]
}

pub fn ship_card_row(count: usize, y: f32, height: f32) -> impl Iterator<Item = Rect> {
    let screen_w = screen_width();
    let gap = (screen_w * 0.022).min(28.0);
    let gaps = gap * count.saturating_sub(1) as f32;
    let width = ((screen_w * 0.94 - gaps) / count.max(1) as f32).min(260.0);
    let left = (screen_w - width * count as f32 - gaps) / 2.0;
    (0..count).map(move |index| Rect::new(left + index as f32 * (width + gap), y, width, height))
}

pub fn render_ship_cards(
    selected: ShipKind,
    y: f32,
    card_height: f32,
    res: &Resources,
) -> Option<ShipKind> {
    let ships: Vec<_> = res.selectable_ships().collect();
    let count = ships.len();
    let mut click = None;
    for (kind, card) in ships.into_iter().zip(ship_card_row(count, y, card_height)) {
        let x = card.x;
        let card_width = card.w;
        if draw_menu_button(card, kind == selected) {
            click = Some(kind);
        }
        if count > 1 && kind == selected {
            let key_width = (card_width * 0.23).min(34.0);
            let left_key = Rect::new(x + 5.0, y + 4.0, key_width, 22.0);
            let right_key = Rect::new(x + card_width - key_width - 5.0, y + 4.0, key_width, 22.0);
            draw_menu_label("←", left_key, 16, LIGHTGRAY, res);
            draw_menu_label("→", right_key, 16, LIGHTGRAY, res);
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
        draw_menu_label(&res.lang.t(kind.label()), label_bounds, 20, WHITE, res);
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

#[derive(Clone, Copy)]
pub enum BriefingAction {
    Launch,
    WaitForHost,
    Hidden,
}

pub fn render_briefing(
    mission: &Mission,
    player_name: &str,
    ships: &[Option<ShipKind>],
    action: BriefingAction,
    res: &Resources,
) -> bool {
    let screen = vec2(screen_width(), screen_height());
    let frame = vec2(960.0, 660.0);
    let scale = (screen / frame).min_element().min(1.2);
    let origin = (screen - frame * scale) / 2.0;
    let rect = |x, y, w, h| {
        Rect::new(
            origin.x + x * scale,
            origin.y + y * scale,
            w * scale,
            h * scale,
        )
    };
    let band = |y, h| rect(16.0, y, 928.0, h);

    draw_menu_label(
        &format!("{} {}", res.lang.t("mission"), mission.level_id),
        band(12.0, 37.0),
        26,
        ORANGE,
        res,
    );
    draw_menu_label(
        &mission_title(mission, res),
        band(47.0, 62.0),
        46,
        WHITE,
        res,
    );
    draw_menu_label(
        &res.lang.t("objectives"),
        band(113.0, 32.0),
        21,
        LIGHTGRAY,
        res,
    );

    for (row, &objective) in mission.objectives.iter().enumerate() {
        let label = res.lang.t(objective_label(objective));
        let text = if objective == MissionObjective::DefeatBoss {
            label.to_string()
        } else {
            format!("{label}  {}", objective.target())
        };
        draw_menu_label(&text, band(145.0 + row as f32 * 32.0, 32.0), 23, WHITE, res);
    }

    let fleet = rect(16.0, 223.0, 320.0, 355.0);
    let threats = rect(416.0, 223.0, 528.0, 355.0);
    for (panel, fill, edge) in [
        (fleet, Color::from_rgba(15, 38, 56, 238), SKYBLUE),
        (threats, Color::from_rgba(91, 53, 49, 242), ORANGE),
    ] {
        draw_rectangle(panel.x, panel.y, panel.w, panel.h, fill);
        draw_rectangle_lines(panel.x, panel.y, panel.w, panel.h, 1.5 * scale, edge);
    }

    draw_menu_label(
        &pilot_label("commander_title", player_name, res),
        rect(16.0, 234.0, 320.0, 36.0),
        21,
        SKYBLUE,
        res,
    );
    draw_menu_label("VS", rect(336.0, 377.0, 80.0, 70.0), 36, WHITE, res);
    draw_menu_label(
        &res.lang.t("briefing_threats"),
        rect(416.0, 234.0, 528.0, 36.0),
        21,
        ORANGE,
        res,
    );
    draw_briefing_fleet(ships, fleet.center() + vec2(0.0, 22.0 * scale), scale, res);
    draw_briefing_threats(
        mission.level_id,
        threats.center() + vec2(0.0, 12.0 * scale),
        scale,
        res,
    );

    match action {
        BriefingAction::Launch => menu_button(
            &res.lang.t("press_space"),
            rect(285.0, 593.0, 390.0, 52.0),
            24,
            false,
            res,
        ),
        BriefingAction::WaitForHost => {
            draw_menu_label(
                &res.lang.t("lan_wait_launch"),
                band(593.0, 52.0),
                20,
                SKYBLUE,
                res,
            );
            false
        }
        BriefingAction::Hidden => false,
    }
}

fn draw_briefing_fleet(ships: &[Option<ShipKind>], center: Vec2, scale: f32, res: &Resources) {
    let ships: Vec<_> = ships.iter().flatten().copied().collect();
    let first_row_y = if ships.len() > 2 { -65.0 } else { 0.0 };
    let size = if ships.len() == 1 { 206.0 } else { 134.0 };
    let elapsed = get_time() as f32;
    for (row, ships) in ships.chunks(2).enumerate() {
        for (column, &kind) in ships.iter().enumerate() {
            let phase = elapsed * 1.4 + (row * 2 + column) as f32 * 1.1;
            let x = (column as f32 - (ships.len() - 1) as f32 / 2.0) * 145.0;
            let y = first_row_y + row as f32 * 130.0 + phase.sin() * 5.0;
            draw_ship_hull(
                kind,
                center + vec2(x, y) * scale,
                size * scale,
                std::f32::consts::FRAC_PI_2 + phase.sin() * 0.04,
                0.2,
                WHITE,
                res,
            );
        }
    }
}

fn draw_briefing_threats(level: u32, center: Vec2, scale: f32, res: &Resources) {
    let possible = [
        EnemyType::Regular,
        EnemyType::Kamikaze,
        EnemyType::Interceptor,
        EnemyType::Heavy,
        EnemyType::Guardian,
    ];
    let (count, size, first_row_y) = match level {
        0..=1 => (2, 156.0, 0.0),
        2..=3 => (3, 156.0, 0.0),
        _ => (5, 112.0, -66.0),
    };
    let slot_width = 164.0 * scale;
    for (row, enemies) in possible[..count].chunks(3).enumerate() {
        for (column, &enemy) in enemies.iter().enumerate() {
            let x = center.x + (column as f32 - (enemies.len() - 1) as f32 / 2.0) * slot_width;
            let y = center.y + (first_row_y + row as f32 * 132.0) * scale;
            enemy_sprite(enemy, res).draw_tinted(
                vec2(x, y),
                size * scale,
                -std::f32::consts::FRAC_PI_2,
                BLACK,
            );
            draw_menu_label(
                &res.lang.t(enemy_name_key(enemy)),
                Rect::new(
                    x - slot_width / 2.0,
                    y + size * 0.5 * scale,
                    slot_width,
                    24.0 * scale,
                ),
                15,
                WHITE,
                res,
            );
        }
    }
}

pub fn render_launch_cutscene(ships: &[Option<ShipKind>], elapsed: f32, res: &Resources) {
    let window = vec2(screen_width(), screen_height());
    let scale = (window / vec2(960.0, 620.0)).min_element().min(1.2);
    let center = vec2(window.x * 0.5, window.y * 0.49);
    let flight = (elapsed / 1.5).clamp(0.0, 1.0);
    let approach = (1.0 - flight).powi(3);
    let reveal = ((elapsed - 0.65) / 0.85).clamp(0.0, 1.0);
    let takeoff = ((elapsed - 3.5) / 1.0).clamp(0.0, 1.0);
    let takeoff_ease = takeoff * takeoff;
    draw_rectangle(
        0.0,
        0.0,
        window.x,
        window.y,
        Color::new(0.0, 0.03, 0.08, 0.86),
    );
    for i in 0..24 {
        let angle = i as f32 * std::f32::consts::TAU / 24.0;
        let direction = vec2(angle.cos(), angle.sin());
        let distance = (i as f32 * 0.37 + elapsed * 1.6).fract();
        let start = center + direction * (80.0 + 240.0 * distance) * scale;
        let end = start + direction * (8.0 + 26.0 * distance) * scale;
        draw_line(
            start.x,
            start.y,
            end.x,
            end.y,
            2.0 * scale,
            Color {
                a: (1.0 - distance) * 0.7,
                ..SKYBLUE
            },
        );
    }
    let ring_radius = (84.0 + 38.0 * reveal) * scale;
    draw_circle(
        center.x,
        center.y,
        ring_radius,
        Color::new(0.06, 0.14, 0.25, 0.8 * reveal),
    );
    draw_circle_lines(
        center.x,
        center.y,
        ring_radius,
        2.0 * scale,
        Color {
            a: 0.65 * reveal,
            ..GOLD
        },
    );

    let kinds: Vec<_> = ships.iter().flatten().copied().collect();
    let spacing = 84.0_f32.min(240.0 / kinds.len().max(1) as f32) * scale;
    for (index, kind) in kinds.iter().enumerate() {
        let offset_x = (index as f32 - (kinds.len().saturating_sub(1) as f32 * 0.5)) * spacing;
        let formation = center + vec2(offset_x, 24.0 * scale);
        let start = center + vec2(-0.85 * window.x, 0.42 * window.y);
        let position =
            formation + (start - formation) * approach - vec2(0.0, window.y * 0.95 * takeoff_ease);
        let alpha = (1.0 - ((takeoff - 0.62) / 0.38).clamp(0.0, 1.0)).clamp(0.0, 1.0);
        let size = (178.0 - 35.0 * approach) * scale;
        let hull_rotation = 0.54 * approach;
        draw_launch_engine(
            position,
            hull_rotation,
            size,
            approach.max(takeoff),
            alpha,
            &res.ship_flame,
        );
        draw_ship_hull(
            *kind,
            position,
            size,
            hull_rotation,
            approach.max(takeoff),
            Color { a: alpha, ..WHITE },
            res,
        );
    }

    if elapsed >= 1.5 {
        let count = (3.0 - (elapsed - 1.5).floor()).clamp(1.0, 3.0) as u32;
        let pulse = 1.0 + ((elapsed - 1.5) * 5.0).sin().abs() * 0.08;
        let text = count.to_string();
        let size = (104.0 * scale * pulse) as u16;
        draw_text_centered(&text, 0.0, size, GOLD, res);
    }
}

pub fn render_enemy_codex(selected: usize, res: &Resources) {
    let entries = [
        EnemyType::Regular,
        EnemyType::Kamikaze,
        EnemyType::Interceptor,
        EnemyType::Heavy,
        EnemyType::Guardian,
    ];
    draw_text_centered(&res.lang.t("enemy_codex"), -275.0, 30, ORANGE, res);
    let list_x = screen_width() * 0.04;
    let list_width = screen_width() * 0.35;
    for (index, enemy) in entries.into_iter().enumerate() {
        let y = screen_height() * 0.28 + index as f32 * 52.0;
        let color = if index == selected { YELLOW } else { LIGHTGRAY };
        draw_menu_label(
            &res.lang.t(enemy_name_key(enemy)),
            Rect::new(list_x, y, list_width, 38.0),
            15,
            color,
            res,
        );
    }
    let selected_enemy = entries[selected.min(entries.len() - 1)];
    enemy_sprite(selected_enemy, res).draw(
        vec2(screen_width() * 0.54, screen_height() * 0.52),
        210.0,
        0.0,
    );
    draw_menu_label(
        &res.lang.t(enemy_name_key(selected_enemy)),
        Rect::new(
            screen_width() * 0.69,
            screen_height() * 0.36,
            screen_width() * 0.28,
            40.0,
        ),
        20,
        YELLOW,
        res,
    );
    draw_menu_label(
        &res.lang.t(enemy_lore_key(selected_enemy)),
        Rect::new(
            screen_width() * 0.68,
            screen_height() * 0.43,
            screen_width() * 0.29,
            150.0,
        ),
        12,
        WHITE,
        res,
    );
    draw_text_centered(&res.lang.t("codex_controls"), 280.0, 14, GRAY, res);
}

fn enemy_name_key(enemy: EnemyType) -> &'static str {
    match enemy {
        EnemyType::Regular => "enemy_regular",
        EnemyType::Kamikaze => "enemy_kamikaze",
        EnemyType::Interceptor => "enemy_interceptor",
        EnemyType::Heavy => "enemy_heavy",
        EnemyType::Guardian => "enemy_guardian",
    }
}

fn enemy_lore_key(enemy: EnemyType) -> &'static str {
    match enemy {
        EnemyType::Regular => "lore_regular",
        EnemyType::Kamikaze => "lore_kamikaze",
        EnemyType::Interceptor => "lore_interceptor",
        EnemyType::Heavy => "lore_heavy",
        EnemyType::Guardian => "lore_guardian",
    }
}

fn enemy_sprite(enemy: EnemyType, res: &Resources) -> &ModelSprite {
    match enemy {
        EnemyType::Regular => &res.enemy_small,
        EnemyType::Kamikaze => &res.enemy_kamikaze,
        EnemyType::Interceptor => &res.enemy_interceptor,
        EnemyType::Heavy => &res.enemy_heavy,
        EnemyType::Guardian => &res.enemy_guardian,
    }
}

pub fn render_mission_success(mission: &Mission, advance: bool, res: &Resources) -> bool {
    draw_text_centered(&res.lang.t("mission_complete"), -50.0, 40, GREEN, res);
    draw_text_centered(&level_cleared_label(mission, res), 10.0, 24, WHITE, res);
    advance && centered_menu_button(&res.lang.t("next_mission"), 100.0, res)
}

fn pilot_label<'a>(key: &str, name: &str, res: &'a Resources) -> Cow<'a, str> {
    let mut args = fluent_bundle::FluentArgs::new();
    args.set("name", name);
    res.lang.format(key, Some(&args))
}

fn mission_title<'a>(mission: &Mission, res: &'a Resources) -> Cow<'a, str> {
    match mission.level_id {
        1 => res.lang.t("mission_dust"),
        2 => res.lang.t("mission_kamikaze"),
        4 => res.lang.t("mission_scrap_yard"),
        5 => res.lang.t("mission_dreadnought"),
        level => level_label("mission_deep_space", level, res),
    }
}

fn level_cleared_label<'a>(mission: &Mission, res: &'a Resources) -> Cow<'a, str> {
    level_label("level_cleared", mission.level_id, res)
}

fn level_label<'a>(key: &str, level: u32, res: &'a Resources) -> Cow<'a, str> {
    let mut args = fluent_bundle::FluentArgs::new();
    args.set("level", level);
    res.lang.format(key, Some(&args))
}

pub fn render_lan_victory(
    mission: &Mission,
    kind: ShipKind,
    player_name: &str,
    points: u32,
    progress: f32,
    res: &Resources,
) {
    let window = vec2(screen_width(), screen_height());
    let scale = (window / vec2(960.0, 620.0)).min_element().min(1.2);
    let center = vec2(window.x * 0.5, window.y * 0.47);
    let flight = (progress / 0.72).min(1.0);
    let approach = (1.0 - flight).powi(3);
    let reveal = ((progress - 0.42) / 0.35).clamp(0.0, 1.0);

    draw_rectangle(
        0.0,
        0.0,
        window.x,
        window.y,
        Color::new(0.0, 0.03, 0.08, 0.46),
    );
    for i in 0..24 {
        let angle = i as f32 * std::f32::consts::TAU / 24.0;
        let direction = vec2(angle.cos(), angle.sin());
        let distance = (i as f32 * 0.37 + progress * 1.6).fract();
        let start = center + direction * (80.0 + 240.0 * distance) * scale;
        let end = start + direction * (8.0 + 26.0 * distance) * scale;
        let alpha = (1.0 - distance) * 0.7 * (1.0 - progress);
        draw_line(
            start.x,
            start.y,
            end.x,
            end.y,
            2.0 * scale,
            Color {
                a: alpha,
                ..SKYBLUE
            },
        );
    }
    let ring_radius = (84.0 + 38.0 * reveal) * scale;
    draw_circle(
        center.x,
        center.y,
        ring_radius,
        Color::new(0.06, 0.14, 0.25, 0.8 * reveal),
    );
    draw_circle_lines(
        center.x,
        center.y,
        ring_radius,
        2.0 * scale,
        Color {
            a: 0.65 * reveal,
            ..GOLD
        },
    );

    draw_ship_hull(
        kind,
        center + vec2(-0.26, 0.58) * window * approach,
        (222.0 - 128.0 * approach) * scale,
        0.54 * approach,
        approach,
        WHITE,
        res,
    );

    let label_bounds = |y, h| Rect::new(0.0, window.y * y, window.x, window.y * h);
    let title_alpha = (progress * 4.0).min(1.0);
    draw_menu_label(
        &res.lang.t("mission_complete"),
        label_bounds(0.11, 0.09),
        40,
        Color {
            a: title_alpha,
            ..GREEN
        },
        res,
    );
    draw_menu_label(
        &level_cleared_label(mission, res),
        label_bounds(0.20, 0.07),
        22,
        Color {
            a: title_alpha,
            ..WHITE
        },
        res,
    );
    let gold = Color { a: reveal, ..GOLD };
    draw_menu_label(
        &res.lang.t("lan_mvp"),
        label_bounds(0.673, 0.045),
        17,
        gold,
        res,
    );
    draw_menu_label(player_name, label_bounds(0.716, 0.077), 34, gold, res);
    draw_menu_label(
        &format!("{} {points}", res.lang.t("lan_mvp_points")),
        label_bounds(0.793, 0.042),
        15,
        Color { a: reveal, ..WHITE },
        res,
    );
}

pub fn render_upgrade_bay(
    pilot: &Pilot,
    selection: &UpgradeSelection,
    continue_key: &str,
    res: &Resources,
) -> Option<usize> {
    draw_background(&res.shop_background);
    let width = screen_width();
    let height = screen_height();
    let start_y = (height * 0.5 - 130.0).max(height * 0.28);
    draw_rectangle(0.0, 0.0, width, height, Color::new(0.015, 0.02, 0.06, 0.62));
    let header_bounds = |top, height| Rect::new(0.0, top * start_y, width, height * start_y);
    draw_menu_label(
        &res.lang.t("upgrade_bay_title"),
        header_bounds(0.12, 0.25),
        40,
        ORANGE,
        res,
    );
    draw_menu_label(
        &res.lang.t("upgrade_bay_subtitle"),
        header_bounds(0.43, 0.18),
        18,
        GRAY,
        res,
    );

    let resources_text = format!(
        "{} {} {} | {} {}",
        res.lang.t("resources"),
        res.lang.t("rust"),
        pilot.ship.scrap,
        res.lang.t("gold"),
        pilot.ship.rare_metal
    );
    draw_menu_label(&resources_text, header_bounds(0.61, 0.18), 22, YELLOW, res);

    let row_count = (UpgradeSelection::CONTINUE_INDEX + 1) as f32;
    let line_height = ((height - start_y - 44.0) / row_count).clamp(1.0, 52.0);
    let row_height = line_height * (46.0 / 52.0);
    let row_width = (width * 0.58).min(680.0);
    let row_x = width * 0.04;
    let mut clicked = None;
    let (mouse_x, mouse_y) = mouse_position();
    let hovered = UPGRADABLE_IDS.iter().enumerate().find_map(|(idx, _)| {
        let y = start_y + line_height * idx as f32;
        Rect::new(row_x, y, row_width, row_height)
            .contains(vec2(mouse_x, mouse_y))
            .then_some(idx)
    });
    let active_index = hovered.unwrap_or(selection.0);
    let active_upgrade = UPGRADABLE_IDS.get(active_index).copied();

    let preview_width = width - row_width - 2.0 * row_x;
    let ship_size = (height * 0.35).min(preview_width * 0.86).min(340.0);
    let ship_pos = vec2(
        row_x + row_width + preview_width / 2.0,
        start_y + line_height * row_count / 2.0,
    );
    draw_ship_hull(pilot.ship.kind, ship_pos, ship_size, 0.0, 0.0, WHITE, res);
    let mark_mount = |offset: Vec2, color: Color| {
        let marker = ship_pos + offset * ship_size;
        draw_circle_lines(marker.x, marker.y, ship_size * 0.075, 3.0, color);
        draw_circle(marker.x, marker.y, ship_size * 0.025, color);
    };
    let dual_mounts = pilot.ship.kind.dual_cannon_mounts();
    match active_upgrade {
        Some(UpgradeId::DualCannons) => {
            for mount in &dual_mounts {
                mark_mount(mount.offset, ORANGE);
            }
        }
        Some(UpgradeId::WeaponTuning) => mark_mount(vec2(0.0, -0.30), ORANGE),
        Some(UpgradeId::ReinforcedHull) => mark_mount(Vec2::ZERO, SKYBLUE),
        Some(UpgradeId::EngineOverdrive) => mark_mount(vec2(0.0, 0.31), YELLOW),
        Some(UpgradeId::MagnetArray) => mark_mount(vec2(0.32, 0.04), GREEN),
        Some(UpgradeId::ShieldCapacitor) => mark_mount(vec2(-0.32, 0.04), VIOLET),
        None => {}
    }

    for (idx, id) in UPGRADABLE_IDS.iter().copied().enumerate() {
        let def = upgrade_definition(id);
        let level = pilot.upgrade_level(id);
        let max_level = def.max_level;
        let selected = active_index == idx;

        let status_text = if level >= max_level {
            res.lang.t("upgrade_status_maxed").to_string()
        } else {
            let next_level = level + 1;
            let cost = crate::game::upgrade_cost(id, next_level);
            let can_buy = pilot.can_buy_upgrade(id);
            let buy_state = if can_buy {
                res.lang.t("upgrade_status_buy")
            } else {
                res.lang.t("upgrade_status_lack")
            };
            format!(
                "{}: {} {} ({})",
                res.lang.t("upgrade_cost"),
                res.lang.t("gold"),
                cost.rare_metal,
                buy_state
            )
        };

        let (display_level, display_max, desc_key) = if id == UpgradeId::DualCannons {
            let desc_key = if dual_mounts[1].rear_facing {
                "upgrade_cannon_rear_desc"
            } else {
                def.desc_key
            };
            (level + 1, max_level + 1, desc_key)
        } else {
            (level, max_level, def.desc_key)
        };
        let row_text = format!(
            "{}  [{} {}/{}]  {}",
            res.lang.t(def.name_key),
            res.lang.t("upgrade_level"),
            display_level,
            display_max,
            status_text
        );

        let y = start_y + line_height * idx as f32;
        let bounds = Rect::new(row_x, y, row_width, row_height);
        if draw_menu_button(bounds, selected) {
            clicked = Some(idx);
        }
        draw_menu_label(
            &row_text,
            Rect::new(row_x, y, row_width, row_height * 0.56),
            20,
            WHITE,
            res,
        );
        draw_menu_label(
            &res.lang.t(desc_key),
            Rect::new(row_x, y + row_height * 0.54, row_width, row_height * 0.40),
            14,
            LIGHTGRAY,
            res,
        );
    }

    let sell_idx = UpgradeSelection::SELL_SCRAP_INDEX;
    let sell_bounds = Rect::new(
        row_x,
        start_y + line_height * sell_idx as f32,
        row_width,
        row_height,
    );
    let sell_text = format!(
        "{} (10 {} = 1 {})",
        res.lang.t("sell_scrap"),
        res.lang.t("rust"),
        res.lang.t("gold")
    );
    if menu_button(&sell_text, sell_bounds, 20, selection.0 == sell_idx, res) {
        clicked = Some(sell_idx);
    }

    let continue_idx = UpgradeSelection::CONTINUE_INDEX;
    let bounds = Rect::new(
        row_x,
        start_y + line_height * continue_idx as f32,
        row_width,
        row_height,
    );
    if menu_button(
        &res.lang.t(continue_key),
        bounds,
        24,
        selection.0 == continue_idx,
        res,
    ) {
        clicked = Some(continue_idx);
    }

    draw_menu_label(
        &res.lang.t("upgrade_controls"),
        Rect::new(row_x, start_y + line_height * row_count, row_width, 28.0),
        16,
        GRAY,
        res,
    );
    clicked
}

pub fn render_game_over(
    score: u32,
    high_score: u32,
    ship: &Ship,
    elapsed: f32,
    retry_available: bool,
    res: &Resources,
) -> (bool, bool) {
    let window = vec2(screen_width(), screen_height());
    let center = ship.pos;
    let scale = (window / vec2(960.0, 620.0)).min_element().min(1.2);
    let fade = ((elapsed - 0.85) / 0.65).clamp(0.0, 1.0);

    draw_rectangle(
        0.0,
        0.0,
        window.x,
        window.y,
        Color::new(0.0, 0.015, 0.035, 0.68),
    );

    if elapsed < 0.3 {
        draw_ship_hull(
            ship.kind,
            center,
            90.0,
            ship.rotation.to_radians() + std::f32::consts::FRAC_PI_2,
            0.0,
            WHITE,
            res,
        );
    }
    let explosion_elapsed = elapsed - 0.22;
    if (0.0..0.4).contains(&explosion_elapsed) {
        let mut explosion = Explosion::new(center, 1.8 * scale);
        explosion.frame = (explosion_elapsed / explosion.frame_time) as usize;
        draw_explosion(&explosion, res);
    }
    let title_alpha = fade;
    draw_menu_label(
        &res.lang.t("game_over"),
        Rect::new(0.0, window.y * 0.13, window.x, window.y * 0.1),
        48,
        Color {
            a: title_alpha,
            ..RED
        },
        res,
    );
    draw_menu_label(
        &format!("{} {}", res.lang.t("final_score_prefix"), score),
        Rect::new(0.0, window.y * 0.25, window.x, window.y * 0.07),
        32,
        Color {
            a: title_alpha,
            ..WHITE
        },
        res,
    );
    draw_menu_label(
        &format!("{} {}", res.lang.t("high_score"), high_score),
        Rect::new(0.0, window.y * 0.32, window.x, window.y * 0.06),
        24,
        Color {
            a: title_alpha,
            ..YELLOW
        },
        res,
    );
    if fade >= 1.0 {
        let retry = retry_available && centered_menu_button(&res.lang.t("restart_run"), 55.0, res);
        let main_menu = centered_menu_button(&res.lang.t("return_to_menu"), 115.0, res);
        (retry, main_menu)
    } else {
        (false, false)
    }
}

pub fn render_pause(res: &Resources) -> (bool, bool) {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.0, 0.0, 0.0, 0.7),
    );

    draw_text_centered(&res.lang.t("paused"), -40.0, 48, YELLOW, res);
    let resume = centered_menu_button(&res.lang.t("press_enter_resume"), 20.0, res);
    let main_menu = centered_menu_button(&res.lang.t("press_esc"), 70.0, res);
    (resume, main_menu)
}
