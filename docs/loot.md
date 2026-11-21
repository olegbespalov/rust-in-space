# Loot

A reference to the loot items currently implemented in the game. Images use the same PNG assets that serve as sprite fallbacks.

| Item | Image | Effect |
|---|---|---|
| Scrap (`Scrap`) | ![Scrap](../assets/loot/resources/rust_pile.png) | Common resource used for upgrades. The amount depends on the drop source. |
| Rare metal (`RareMetal`) | ![Rare metal](../assets/loot/resources/gold.png) | Rare resource used for major upgrades. |
| Health pack (`HealthPack`) | ![Health pack](../assets/loot/health.png) | Restores 25 health. |
| Rapid fire (`RapidFireBoost`) | ![Rapid fire](../assets/loot/energy.png) | Temporarily reduces the interval between shots. |
| Big bullets (`BigBulletBoost`) | ![Big bullets](../assets/loot/bigger-ammo.png) | Temporarily increases projectile damage and size. |
| Magnetic ammo (`MagneticAmmo`) | ![Magnetic ammo](../assets/loot/magnetic-ammo.png) | Player projectiles begin homing toward enemies. |
| Shield (`Shield`) | ![Shield](../assets/loot/shield.png) | Adds a separate durability pool; drops contain 30–100 HP. |
| Invisibility (`Invisibility`) | ![Mystery item](../assets/loot/mystery.png) | Temporarily makes the ship invisible. |

Loot drop chances vary by source and are modified by the selected difficulty. Destroying a regular enemy can also grant magnetic ammo or invisibility as an extra drop. Rare asteroids always drop an item, and bosses always drop rare metal.
