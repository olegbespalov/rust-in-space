# Ship models and upgrades

The player flies a weathered white-and-blue fighter with an orange cockpit. The player hull uses a [textured Meshy GLB](assets/models/player_ship.glb) generated from [the original sprite](assets/ship_body.png). Macroquad renders the mesh from a shallow overhead angle in a 72-pixel drawing area. Thrust tips the hull to expose more of its rear profile. The original sprite is the fallback.

## Player upgrades

The Upgrade Bay already sells these upgrades for scrap and rare metal between missions. Purchased levels carry through the current run. Start with the proposed flight cues below. A future Upgrade Bay preview can show details that are hard to see in combat.

- **Reinforced Hull (3 levels)**. Show progressively heavier shoulder plates and bracing. Keep the nose, cockpit, and wing outline recognizable.
- **Weapon Tuning (3 levels)**. Build up the existing nose assembly with a thicker collar and heat sinks. Keep the muzzle aligned with the ship heading.
- **Engine Overdrive (3 levels)**. Add more substantial rear housings and brighter exhaust rings. Match the nozzles to the existing flame position.
- **Magnet Array (2 levels)**. Add pickup coils and a short pulse when nearby loot is pulled in.
- **Shield Capacitor (1 level)**. Add a compact emitter to the hull. Use the existing shield effect while the shield is active.

Use the purchased upgrade level to select each part or effect. Author one hull and reusable parts that share a pivot. Preview individual upgrades and a fully upgraded ship to check that the parts fit together.

## Enemy variants

The regular shooter, kamikaze, Guardian, interceptor, and heavy gunship belong to the same civilization in the first world. Their ships share segmented green armor, violet chitin spikes, and red eye-like sensors. Distinguish combat roles by silhouette and proportions. Give the second world's civilization its own hull shapes, materials, and palette so players can recognize the change in faction at a glance.

1. **Interceptor (48 px)**. An elongated fighter with a forked nose and swept-back violet spines, derived from the [regular shooter sprite](assets/enemy.png). [Meshy GLB](assets/models/enemy_interceptor.glb), task `01a1fef7-fbff-76d4-a1fe-12f2b74d0a8a`.
2. **Heavy enemy (72 px)**. A wide gunship with green armor, red seams, and a single eye above three gun pods, derived from the [Guardian sprite](assets/boss_1.png). [Meshy GLB](assets/models/enemy_heavy.glb), task `01a1fef8-18a1-7399-88bd-6eb5033417be`.

Both meshes are oriented with the nose at +Y and the top at +Z. The embedded 512-pixel PNG textures were brightened for the dark space background. The heavy gunship's textured eye was fitted into the forward recess to keep it visible at gameplay scale. Macroquad renders the meshes once at startup and rotates their baked textures during play. Meshy did not return license metadata for either task.

## Meshy production workflow

1. Use the current sprite as the reference for a base model. For an upgrade, use a reference that shows the same hull with the proposed part fitted. Check the top, rear, and underside before accepting an export.
2. Generate a textured GLB using Smart Topology and target up to 4,000 triangles for a small ship. Check the silhouette at its in-game size before refining surface detail. Keep the embedded base-color texture and bake transforms before export.
3. Align the hull center to the gameplay pivot, the nose to **+Y**, and the top to **+Z**. Align weapon, exhaust, and upgrade attachment positions to that same pivot. The current player loader accepts one indexed triangle primitive with normals, UVs, and an embedded base-color image.
4. Review the model against the space background while idle, turning, and thrusting. Check the flame and shield alignment on the player and the health bar placement on enemies. Test with multiple enemies on screen before choosing how to render the whole fleet.
5. Record the source artwork, Meshy task ID, and asset license with each accepted export. The first player model is Meshy task `01a1dfac-c700-718d-89a3-36263032c0fc`.

The visual upgrades and Upgrade Bay model preview are proposed work.

## Asteroids

The ordinary and rare asteroids use textured low-poly Meshy models, rendered once to textures at startup. Their original sprites remain fallbacks. The models were generated from text descriptions; the rare model includes mineral deposits. Meshy texture task IDs are `01a1dfc9-3012-74d2-ab9b-7026527e854c` (ordinary) and `01a1dfc9-1bee-7496-a6d4-61379c69b20e` (rare). The embedded JPEG textures were converted to PNG for the game's existing image decoder. Meshy did not return license metadata for these tasks.
