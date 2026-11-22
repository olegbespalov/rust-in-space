# Ship models and upgrades

The player chooses a ship before the first mission briefing. Both ships share the same flight and combat stats. Macroquad renders the selected mesh from a shallow overhead angle in a 90-pixel drawing area. Thrust tips the hull to expose more of its rear profile.

- **Alpha** is the weathered white-and-blue fighter with an orange cockpit. Its [textured Meshy GLB](assets/models/player_ship.glb) was generated from [the original sprite](assets/ship_body.png), which also serves as the fallback.
- **Delta** is the copper-red fighter with ivory wing panels and a pale cyan cockpit. Its [textured Meshy GLB](assets/models/player_delta.glb) was generated from a text prompt, with 3,682 triangles and an embedded 512-pixel base-color texture. The textured Meshy task is `01a1ffd1-213f-767c-bc8e-dbd9c79e3f03`. Meshy returned no license metadata.

## Player upgrades

The Upgrade Bay already sells these upgrades for scrap and rare metal between missions. Purchased levels carry through the current run. Start with the proposed flight cues below. A future Upgrade Bay preview can show details that are hard to see in combat.

- **Reinforced Hull (3 levels)**. Show progressively heavier shoulder plates and bracing. Keep the nose, cockpit, and wing outline recognizable.
- **Weapon Tuning (3 levels)**. Build up the existing nose assembly with a thicker collar and heat sinks. Keep the muzzle aligned with the ship heading.
- **Engine Overdrive (3 levels)**. Add more substantial rear housings and brighter exhaust rings. Match the nozzles to the existing flame position.
- **Magnet Array (2 levels)**. Add pickup coils and a short pulse when nearby loot is pulled in.
- **Shield Capacitor (1 level)**. Add a compact emitter to the hull. Use the existing shield effect while the shield is active.

Use the purchased upgrade level to select each part or effect. Author one hull and reusable parts that share a pivot. Preview individual upgrades and a fully upgraded ship to check that the parts fit together.

## Enemy variants

The regular shooter, kamikaze, Guardian, interceptor, heavy gunship, and Dreadnought belong to the same civilization in the first world. Their ships share segmented green armor, violet chitin spikes, and red eye-like sensors. Distinguish combat roles by silhouette and proportions. Give the second world's civilization its own hull shapes, materials, and palette so players can recognize the change in faction at a glance.

1. **Interceptor (60 px)**. An elongated fighter with a forked nose and swept-back violet spines, derived from the [regular shooter sprite](assets/enemy.png). [Meshy GLB](assets/models/enemy_interceptor.glb), task `01a1fef7-fbff-76d4-a1fe-12f2b74d0a8a`.
2. **Heavy enemy (90 px)**. A wide gunship with green armor, red seams, and a single eye above three gun pods, derived from the [Guardian sprite](assets/boss_1.png). [Meshy GLB](assets/models/enemy_heavy.glb), task `01a1fef8-18a1-7399-88bd-6eb5033417be`.

Both meshes are oriented with the nose at +Y and the top at +Z. The embedded 512-pixel PNG textures were brightened for the dark space background. The heavy gunship's textured eye was fitted into the forward recess to keep it visible at gameplay scale. Macroquad renders the meshes once at startup and rotates their baked textures during play. Meshy did not return license metadata for either task.

The [Dreadnought sprite](assets/boss_dreadnought.png) was generated with ImageGen and renders at 240 pixels. Its two cannon muzzles and side launchers align with the firing warnings. The Guardian uses the former boss models, switching to the damaged hull at half health.

> Create a single transparent game sprite for the final alien capital ship in a retro 2D space shooter. Directly overhead, nose straight up. Broad armored silhouette with two visibly separate forward cannons, layered emerald chitin, violet spines and a bright amber-red reactor. Crisp low-poly shading and bold shapes legible at 240 pixels against dark space. Center the whole ship, fill the square frame with a narrow margin, and leave true transparent alpha around it. No exhaust, detached effects, backdrop, or text.

## Sprite-based Meshy assets

The regular fighter, kamikaze, Guardian, organic projectile, and pickups use Meshy geometry generated from sprite references. Colors from the original sprite are projected onto the model's visible surfaces to preserve faction markings and pickup emblems. Macroquad bakes each model into a small texture at startup and samples it with nearest-neighbor filtering for crisp pixel edges. The source PNG serves as the fallback if the GLB cannot load.

- Regular fighter, kamikaze, Guardian and organic projectile use [enemy.png](assets/enemy.png), [enemy_kamikaze.png](assets/enemy_kamikaze.png), [boss_1.png](assets/boss_1.png) and [enemy_bullet.png](assets/enemy_bullet.png). Their Meshy tasks are `01a1ff40-989e-736a-9fd3-f5d1faa1e49c`, `01a1ff40-aa91-707c-b8a5-60cb39d65cbb`, `01a1ff40-e152-75bb-a821-d4f317ef5290` and `01a1ff42-1546-77bb-b82f-79e28fe3becd`.
- Scrap and gold use [rust_pile.png](assets/loot/resources/rust_pile.png) and [gold.png](assets/loot/resources/gold.png). Their Meshy tasks are `01a1ff40-efcb-75c1-802a-78df3717d3d1` and `01a1ff40-d15d-7540-9ed6-6ecb9f2e21ba`.
- The health, rapid-fire, big-bullet and shield crates use the matching [loot PNGs](assets/loot/). Their Meshy tasks are `01a1ff41-e83b-735d-86b1-d3851cb1e396`, `01a1ff42-21b8-7103-84e6-d03c0cd50739`, `01a1ff42-063c-72b1-9b85-d62981c81c34` and `01a1ff41-f428-77a8-b173-741628998c3d`.
- Magnetic ammo and invisibility use their existing Meshy meshes with colors and icons from [magnetic-ammo.png](assets/loot/magnetic-ammo.png) and [mystery.png](assets/loot/mystery.png).

Meshy did not supply license metadata for the new tasks.

## Meshy production workflow

1. Use the current sprite as the reference for a base model. For an upgrade, use a reference that shows the same hull with the proposed part fitted. Check the top, rear, and underside before accepting an export.
2. Generate a textured GLB using Smart Topology and target up to 4,000 triangles for a small ship. Check the silhouette at its in-game size before refining surface detail. Keep the embedded base-color texture and bake transforms before export.
3. Align the hull center to the gameplay pivot, the nose to **+Y**, and the top to **+Z**. Align weapon, exhaust, and upgrade attachment positions to that same pivot. The current player loader accepts one indexed triangle primitive with normals, UVs, and an embedded base-color image.
4. Review the model against the space background while idle, turning, and thrusting. Check the flame and shield alignment on the player and the health bar placement on enemies. Test with multiple enemies on screen before choosing how to render the whole fleet.
5. Record the source artwork, Meshy task ID, and asset license with each accepted export. The first player model is Meshy task `01a1dfac-c700-718d-89a3-36263032c0fc`.

The visual upgrades and Upgrade Bay model preview are proposed work.

## Asteroids

The ordinary and rare asteroids use low-poly Meshy meshes with crater colors projected from the [ordinary](assets/asteroid.png) and [rare](assets/rare_asteroid.png) pixel-art sprites. The projection uses a reduced palette that keeps the rare asteroid's gold and turquoise deposits visible. Macroquad bakes each mesh into a 40×40 texture and enlarges it with nearest-neighbor filtering. The source sprites serve as fallbacks.

The models were generated from text descriptions. Meshy texture task IDs are `01a1dfc9-3012-74d2-ab9b-7026527e854c` (ordinary) and `01a1dfc9-1bee-7496-a6d4-61379c69b20e` (rare). Meshy did not return license metadata for these tasks.
