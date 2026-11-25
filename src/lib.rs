//! YUNG's Extras: swamp ruins and desert decorations, placed by the world generation from data.
//!
//! Each of the original's Java feature types placed one template centred on the surface after a
//! few ground checks; here every piece is a `minecraft:template` feature
//! (`data/yungsextras/worldgen/feature/`) whose placed feature carries those checks as block
//! predicates (`worldgen/placed_feature/`), and the biome modifiers add them to swamps and
//! deserts (`neoforge/biome_modifier/`). The code registers the three structure processor types
//! the features' processor lists use (`processors.rs`). The tests check that the data hangs together.
use mod_api::{self, Hooks, Registrar, RegistrationContext};
use std::collections::HashMap;
use std::sync::Arc;

mod processors;

#[derive(Default)]
struct Mod;

#[mod_api::sunrise_mod]
impl Hooks for Mod {
    fn register(&self, r: &mut dyn Registrar, _: &RegistrationContext) {
        let keys: processors::Keys = Arc::new(
            processors::BLOCKS
                .iter()
                .map(|id| (*id, r.block_key(id)))
                .collect::<HashMap<_, _>>(),
        );
        r.register_structure_processor_type("pick", processors::PickType(keys.clone()));
        r.register_structure_processor_type("count_cap", processors::CountCapType(keys.clone()));
        r.register_structure_processor_type("yung_pillar", processors::YungPillarType(keys));
    }
}

#[cfg(test)]
mod data_tests;
