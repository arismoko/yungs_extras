# Licence notice: YUNG's Extras for Sunrise

## Original

- **Mod:** YUNG's Extras (`yungsextras`) 1.21.1-NeoForge-5.1.1 by YUNGNICKYOUNG
- **Source:** <https://github.com/YUNG-GANG/YUNGs-Extras>
- **Licence:** LGPL-3.0-only. `LICENSE` is the original's licence text.

## What each file in this package is

| Files | Origin |
|---|---|
| `data/yungsextras/structure/**` (62 `.nbt`) | the original's templates, unchanged |
| `data/yungsextras/loot_table/desert/*.json` (3) | the original's loot tables, updated to Minecraft 26.3's format |
| `data/yungsextras/tags/worldgen/biome/has_structure/*.json` (3) | the original's biome tags, unchanged |
| `data/yungsextras/neoforge/biome_modifier/*.json` (3) | the original's biome modifiers, unchanged |
| `data/yungsextras/worldgen/feature/**`, `worldgen/placed_feature/**` (62 each) | written by `tools/gen_worldgen.py` from the original's configured and placed features |
| `src/`, `tools/`, `Cargo.toml`, `sunrise-mod.toml`, `README.md`, `page.toml` | this port's own work |

This is a modified version of the original, as the LGPL asks to be stated: the original's
eleven Java feature types and its placement modifier are not used. Each piece is a vanilla
template feature, the original's ground checks are block-predicate placement steps, and its
template processors are written as processor data.
