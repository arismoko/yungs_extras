//! YUNG's Extras: swamp ruins and desert decorations, placed by the world generation from data.
//!
//! The mod has no code to run. Each of the original's Java feature types placed one template
//! centred on the surface after a few ground checks; here every piece is a `minecraft:template`
//! feature (`data/yungsextras/worldgen/feature/`) whose placed feature carries those checks as
//! block predicates (`worldgen/placed_feature/`), and the biome modifiers add them to swamps and
//! deserts (`neoforge/biome_modifier/`). The tests check that the data hangs together.
use mod_api::{self, Hooks};

#[derive(Default)]
struct Mod;

#[mod_api::sunrise_mod]
impl Hooks for Mod {}

#[cfg(test)]
mod data_tests;
