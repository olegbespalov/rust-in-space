# Enemies

A reference to the enemies currently implemented in the game. Images use the same PNG assets that serve as sprite fallbacks.

| Enemy | Image | Health | Points | Speed | Attacks and traits |
|---|---|---:|---:|---:|---|
| Regular ship (`Regular`) | ![Regular enemy](../assets/enemy.png) | 24 | 240 | 120 | Fires a single aimed projectile about every 2 seconds. |
| Kamikaze (`Kamikaze`) | ![Kamikaze enemy](../assets/enemy_kamikaze.png) | 18 | 180 | 120 | Rushes the player at 180 units per second while tracking the ship and explodes on contact. |
| Interceptor (`Interceptor`) | ![Interceptor enemy](../assets/enemy.png) | 15 | 150 | 210 | Strafes quickly and fires fast aimed shots about every 2.4 seconds. It uses a separate 3D model. |
| Heavy ship (`Heavy`) | ![Heavy enemy](../assets/boss_1.png) | 60 | 600 | 75 | Fires a spread of three projectiles about every 3.2 seconds. It uses a separate 3D model. |
| Guardian (`Guardian`) | ![Guardian](../assets/boss_1.png) | 105 | 1,050 | 65 | Fires a spread of five aimed shots about every 3.6 seconds. Its hull changes appearance at half health. |
| Dreadnought (`Boss`) | [Dreadnought sprite](../assets/boss_dreadnought.png) | 600 | 6,000 | — | The boss in mission 3. Unlocks missiles at two-thirds health and staggered projectile rings at one-third. Always drops rare metal. |

Kamikazes are available from the start and interceptors appear from level 2. Heavy ships and Guardians appear from level 4. Guardians account for 5% of random spawns when none is already on the field. Every fourth enemy spawn supplies the targeted ship type while an interceptor or heavy-ship objective is still active.

The Dreadnought telegraphs its cannons with orange firing lines, its missiles with blue launch trails, and its rings with a violet radial warning. Player shots can intercept the missiles.

1. Above 400 HP the Dreadnought fires twin cannon volleys and calls in regular fighters.
2. At 400 HP it adds homing missiles, calls in interceptors, and generates a 55 HP shield every 12 seconds.
3. At 200 HP it adds three staggered projectile rings per barrage and calls in interceptors and kamikazes together. Its shield rises to 75 HP and recharges every 8 seconds.

Phase thresholds scale with maximum HP in co-op. The first shield is ready three seconds into phases two and three. Shields expire after four seconds or when their HP runs out.

Enemy health bars show remaining HP. The speed column gives entry speeds in game units per second. Firing intervals are approximate.
