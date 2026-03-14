pub const ROTATION_SPEED: f32 = 200.0;
pub const ACCELERATION: f32 = 150.0;
pub const BULLET_SPEED: f32 = 400.0;
pub const BULLET_LIFETIME: f32 = 2.0;
pub const SHOOT_COOLDOWN: f32 = 0.3;
pub const PLAYER_BULLET_DAMAGE: f32 = 15.0;
pub const PLAYER_BULLET_RADIUS: f32 = 6.0;
pub const BIG_BULLET_DAMAGE: f32 = 30.0;
pub const BIG_BULLET_RADIUS: f32 = 12.0;
pub const ENEMY_BULLET_DAMAGE: f32 = 15.0;
pub const BASE_ASTEROID_DAMAGE: f32 = 5.0;
pub const BASE_KAMIKAZE_DAMAGE: f32 = 30.0;
pub const SCORE_PER_ENEMY_HP: u32 = 10;
pub const ENEMY_BORDER_MARGIN: f32 = 40.0;
pub const ENEMY_HINT_DELAY: f64 = 2.5;
pub const BASE_SHIP_HP: f32 = 150.0;
pub const BASE_LOOT_MAGNET_RADIUS: f32 = 150.0;
pub const HULL_HP_PER_LEVEL: f32 = 20.0;
pub const WEAPON_DAMAGE_BONUS_PER_LEVEL: f32 = 0.10;
pub const ENGINE_ACCEL_BONUS_PER_LEVEL: f32 = 0.08;
pub const MAGNET_RADIUS_PER_LEVEL: f32 = 25.0;
pub const SHIELD_CAPACITOR_HP: f32 = 45.0;
pub const SHIELD_CAPACITOR_DURATION: f32 = 20.0;

pub const BOSS_SHOOT_INTERVAL: f32 = 3.0;
pub const BOSS_BURST_SIZE: u32 = 3;
pub const BOSS_BURST_DELAY: f32 = 0.12;
pub const BOSS_BULLET_SPEED: f32 = 250.0;
pub const BOSS_BORDER_MARGIN: f32 = 70.0;
pub const BOSS_ROTATION_SPEED: f32 = 35.0;
pub const BOSS_HITBOX_RADIUS: f32 = 55.0;

pub const BOSS_VOLLEY_DIRS: [(f32, f32); 4] = [
    (0.0, -1.0), // up
    (0.0, 1.0),  // down
    (-1.0, 0.0), // left
    (1.0, 0.0),  // right
];

// Entity sizes and collision radii
pub const SHIP_RADIUS: f32 = 20.0;
pub const ENEMY_SMALL_RADIUS: f32 = 30.0;
pub const KAMIKAZE_RADIUS: f32 = 22.5;
pub const LARGE_ASTEROID_RADIUS: f32 = 40.0;
pub const MIN_ASTEROID_FOR_FRAGMENTATION: f32 = 15.0;

// Loot constants
pub const LOOT_MAGNET_SPEED: f32 = 300.0;
pub const LOOT_VEL_DECAY: f32 = 0.95;
pub const RAPID_FIRE_DURATION: f32 = 10.0;
pub const BIG_BULLET_DURATION: f32 = 15.0;
pub const SHIELD_DURATION: f32 = 30.0;
