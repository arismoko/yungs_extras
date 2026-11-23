# YUNG's Extras for Sunrise

Swamp ruins and desert decorations: arches, pillars, a ruined church, wells with suspicious sand, obelisks and more.

A port of [YUNG's Extras](https://github.com/YUNG-GANG/YUNGs-Extras) by YUNGNICKYOUNG to [Sunrise](https://playsunrise.net) (Minecraft Java 26.3). The original places its pieces from Java features; here each is a template feature with the original's ground checks written as placement data, so the mod has no code to run.

## What it does

- Swamps: 11 ruined arches, 22 double arches, 3 pillars, 8 cubbies, a ruined church and an ogre's hut, with mossy and cracked bricks, ruined stairs, brick supports down to the ground and coloured candles (one in ten lit).
- Deserts: 9 wells (plain, dry and wishing wells in three sizes), whose sand hides suspicious sand with vanilla's desert-well loot, and in wishing wells the mod's own extra loot; 4 obelisks, a giant torch, small ruins and a chillzone with its loot chest.
- The original's rarity, biome tags and ground checks: pieces only go on sand in deserts, and nothing hangs over a cliff.
- Vanilla's desert well no longer generates where the mod's wells do, as in the original.

## Differences from the original

- Suspicious sand in wells is capped at three on average; the original draws a cap of two to four per well.

## Building

With the Sunrise mod tool (`cargo sunrise-mod`, from the Sunrise SDK) and Rust:

```sh
cargo sunrise-mod build    # compiles against the pinned game, packs target/sunrise-mod/yungsextras.smod
cargo sunrise-mod run      # builds it and starts the game with it
cargo test
```

`sunrise-mod.toml` pins the Sunrise Dev build `dev-2026-11-22-7cfdf81` (game commit `7cfdf81463`). Sunrise builds native mods from source on the player's computer, against the exact game build.

## Licence and credits

YUNG's Extras is by YUNGNICKYOUNG (https://github.com/YUNG-GANG/YUNGs-Extras). Its templates, features and strings are theirs and are used here under the **GNU LGPL v3.0 only**, the original's licence; `LICENSE` holds the text and `LICENSE-NOTICE.md` says what this port carries.

Carried to Minecraft 26.3 by AdLf0208 ([AdLf0208/yungs_extras](https://github.com/AdLf0208/yungs_extras)), and ported to Sunrise by arismoko. This port, its Rust included, is distributed under the same licence.
