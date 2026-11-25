//! The three structure processor types the ruins' processor lists use, registered as
//! `yungsextras:pick`, `yungsextras:count_cap` and `yungsextras:yung_pillar`.
//!
//! * `pick`: a placeholder block becomes one of several states, drawn with one
//!   `settings.getRandom(pos).nextFloat()` against cumulative `below` bounds.
//! * `count_cap`: of the listed blocks in a piece, the first `min` stay, later ones stay with
//!   `chance` until `max` have, and the rest become `replacement` (the wells' suspicious sand).
//! * `yung_pillar`: YUNG's PillarProcessor. A target block becomes a draw of
//!   `target_block_output`, and a column of `pillar_states` runs down from it through air and water.
use mod_api::{
    BlockKey, BlockMetadata, BlockState, BlockUpdateFlags, LegacyRandomSource, ProcessorContext,
    ProcessorDecodeContext, ProcessorFinalizeContext, RandomSource, StructureBlockInfo,
    StructureError, StructureProcessor, StructureProcessorType, Tag,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Every block the processor lists name, requested once at registration.
pub(crate) const BLOCKS: [&str; 19] = [
    "minecraft:black_candle",
    "minecraft:brown_candle",
    "minecraft:brown_stained_glass",
    "minecraft:candle",
    "minecraft:cracked_stone_bricks",
    "minecraft:gray_candle",
    "minecraft:gray_stained_glass",
    "minecraft:green_candle",
    "minecraft:light_gray_candle",
    "minecraft:light_gray_stained_glass",
    "minecraft:mossy_stone_brick_stairs",
    "minecraft:mossy_stone_bricks",
    "minecraft:purple_candle",
    "minecraft:sand",
    "minecraft:stone_brick_stairs",
    "minecraft:stone_bricks",
    "minecraft:suspicious_sand",
    "minecraft:white_candle",
    "minecraft:yellow_stained_glass",
];

/// The registered block keys, by name.
pub(crate) type Keys = Arc<HashMap<&'static str, BlockKey>>;

fn error(message: impl Into<String>) -> StructureError {
    StructureError::new(std::io::Error::other(message.into()))
}

fn field<'a>(config: &'a Tag, key: &str) -> Result<&'a Tag, StructureError> {
    config
        .get(key)
        .ok_or_else(|| error(format!("missing field {key}")))
}

fn number(tag: &Tag, what: &str) -> Result<f64, StructureError> {
    match tag {
        Tag::Double(bits) => Ok(f64::from_bits(*bits)),
        Tag::Float(bits) => Ok(f64::from(f32::from_bits(*bits))),
        Tag::Byte(n) => Ok(f64::from(*n)),
        Tag::Short(n) => Ok(f64::from(*n)),
        Tag::Int(n) => Ok(f64::from(*n)),
        Tag::Long(n) => Ok(*n as f64),
        _ => Err(error(format!("{what} is not a number"))),
    }
}

fn list<'a>(tag: &'a Tag, what: &str) -> Result<&'a [Tag], StructureError> {
    match tag {
        Tag::List(items) => Ok(items),
        _ => Err(error(format!("{what} is not a list"))),
    }
}

fn text(tag: &Tag, what: &str) -> Result<String, StructureError> {
    match tag {
        Tag::String(s) => Ok(s.to_string_lossy()),
        _ => Err(error(format!("{what} is not a string"))),
    }
}

fn key(keys: &Keys, id: &str) -> Result<BlockKey, StructureError> {
    keys.get(id)
        .copied()
        .ok_or_else(|| error(format!("block {id} is not one yungsextras registered")))
}

/// `{"id": "minecraft:candle", "properties": {"candles": "2"}}`.
fn state(
    cx: &dyn ProcessorDecodeContext,
    keys: &Keys,
    tag: &Tag,
) -> Result<BlockState, StructureError> {
    let block = key(keys, &text(field(tag, "id")?, "id")?)?;
    let Some(Tag::Compound(properties)) = tag.get("properties") else {
        return Ok(cx.state(block));
    };
    let pairs = properties
        .iter()
        .map(|(name, value)| Ok((name.to_string_lossy(), text(value, "property")?)))
        .collect::<Result<Vec<_>, StructureError>>()?;
    let pairs: Vec<(&str, &str)> = pairs
        .iter()
        .map(|(n, v)| (n.as_str(), v.as_str()))
        .collect();
    Ok(cx.state_with(block, &pairs))
}

fn block_of(keys: &Keys, tag: &Tag, what: &str) -> Result<BlockKey, StructureError> {
    key(keys, &text(tag, what)?)
}

pub(crate) struct PickType(pub(crate) Keys);
pub(crate) struct CountCapType(pub(crate) Keys);
pub(crate) struct YungPillarType(pub(crate) Keys);

/// A pick's result: one of its states, drawn with `nextInt(len)` when there are several.
#[derive(Debug)]
struct Outcome {
    states: Vec<BlockState>,
    nbt: Option<mod_api::Compound>,
}

impl Outcome {
    fn decode(
        cx: &dyn ProcessorDecodeContext,
        keys: &Keys,
        tag: &Tag,
    ) -> Result<Self, StructureError> {
        let states = match tag.get("states") {
            Some(states) => list(states, "states")?
                .iter()
                .map(|s| state(cx, keys, s))
                .collect::<Result<Vec<_>, _>>()?,
            None => vec![state(cx, keys, field(tag, "state")?)?],
        };
        if states.is_empty() {
            return Err(error("pick states is empty"));
        }
        let nbt = match tag.get("nbt") {
            Some(Tag::Compound(nbt)) => Some(nbt.clone()),
            Some(_) => return Err(error("pick nbt is not a compound")),
            None => None,
        };
        Ok(Self { states, nbt })
    }
}

#[derive(Debug)]
struct Pick {
    block: BlockKey,
    picks: Vec<(f32, Outcome)>,
    default: Option<Outcome>,
    keep_properties: bool,
}

impl StructureProcessorType for PickType {
    fn decode(
        &self,
        cx: &dyn ProcessorDecodeContext,
        config: &Tag,
    ) -> Result<Arc<dyn StructureProcessor>, StructureError> {
        let picks = match config.get("picks") {
            Some(picks) => list(picks, "picks")?
                .iter()
                .map(|p| {
                    let below = number(field(p, "below")?, "below")? as f32;
                    Ok((below, Outcome::decode(cx, &self.0, p)?))
                })
                .collect::<Result<_, StructureError>>()?,
            None => Vec::new(),
        };
        Ok(Arc::new(Pick {
            block: block_of(&self.0, field(config, "block")?, "block")?,
            picks,
            default: config
                .get("default")
                .map(|d| Outcome::decode(cx, &self.0, d))
                .transpose()?,
            keep_properties: matches!(config.get("keep_properties"), Some(Tag::Byte(1))),
        }))
    }
}

/// `Block.withPropertiesOf` for the properties a stair has.
fn keep_properties(
    metadata: &dyn BlockMetadata,
    from: BlockState,
    mut to: BlockState,
) -> BlockState {
    for name in ["facing", "half", "shape", "waterlogged"] {
        if let Ok(Some(value)) = metadata.property(from, name)
            && let Ok(next) = metadata.with(to, name, value)
        {
            to = next;
        }
    }
    to
}

impl StructureProcessor for Pick {
    fn process_block(
        &self,
        cx: &mut dyn ProcessorContext,
        _original: &StructureBlockInfo,
        mut current: StructureBlockInfo,
    ) -> Result<Option<StructureBlockInfo>, StructureError> {
        if current.state.block() != self.block {
            return Ok(Some(current));
        }
        let outcome = if self.picks.is_empty() {
            self.default.as_ref()
        } else {
            let draw = cx.random().next_float();
            self.picks
                .iter()
                .find(|(below, _)| draw < *below)
                .map(|(_, outcome)| outcome)
                .or(self.default.as_ref())
        };
        let Some(outcome) = outcome else {
            return Ok(Some(current));
        };
        let picked = match outcome.states.as_slice() {
            [one] => *one,
            many => many[cx.random().next_int_bound(many.len() as i32) as usize],
        };
        current.state = match self.keep_properties {
            true => keep_properties(cx.metadata(), current.state, picked),
            false => picked,
        };
        if let Some(nbt) = &outcome.nbt {
            current.nbt = Some(nbt.clone());
        }
        Ok(Some(current))
    }
}

#[derive(Debug)]
struct CountCap {
    blocks: Vec<BlockKey>,
    min: u32,
    max: u32,
    chance: f32,
    replacement: BlockState,
}

impl StructureProcessorType for CountCapType {
    fn decode(
        &self,
        cx: &dyn ProcessorDecodeContext,
        config: &Tag,
    ) -> Result<Arc<dyn StructureProcessor>, StructureError> {
        Ok(Arc::new(CountCap {
            blocks: list(field(config, "blocks")?, "blocks")?
                .iter()
                .map(|b| block_of(&self.0, b, "blocks entry"))
                .collect::<Result<_, _>>()?,
            min: number(field(config, "min")?, "min")? as u32,
            max: number(field(config, "max")?, "max")? as u32,
            chance: number(field(config, "chance")?, "chance")? as f32,
            replacement: state(cx, &self.0, field(config, "replacement")?)?,
        }))
    }
}

impl StructureProcessor for CountCap {
    fn finalize_processing(
        &self,
        cx: &mut dyn ProcessorFinalizeContext,
        _original: &[StructureBlockInfo],
        mut processed: Vec<StructureBlockInfo>,
    ) -> Result<Vec<StructureBlockInfo>, StructureError> {
        let mut counted = 0;
        for info in &mut processed {
            if !self.blocks.contains(&info.state.block()) {
                continue;
            }
            let keep = counted < self.min
                || (counted < self.max && cx.random_at(info.pos).next_float() <= self.chance);
            if keep {
                counted += 1;
            } else {
                info.state = self.replacement;
                info.nbt = None;
            }
        }
        Ok(processed)
    }
}

/// A weighted list of states: each entry owns a share of one `nextFloat`, the rest is the default.
#[derive(Debug)]
struct Randomizer {
    entries: Vec<(BlockState, f32)>,
    default: BlockState,
}

impl Randomizer {
    fn decode(
        cx: &dyn ProcessorDecodeContext,
        keys: &Keys,
        tag: &Tag,
    ) -> Result<Self, StructureError> {
        Ok(Self {
            entries: list(field(tag, "entries")?, "entries")?
                .iter()
                .map(|e| {
                    Ok((
                        state(cx, keys, field(e, "blockState")?)?,
                        number(field(e, "probability")?, "probability")? as f32,
                    ))
                })
                .collect::<Result<_, StructureError>>()?,
            default: state(cx, keys, field(tag, "defaultBlockState")?)?,
        })
    }

    fn get(&self, random: &mut dyn RandomSource) -> BlockState {
        let target = random.next_float();
        let mut bottom = 0.0;
        for &(state, probability) in &self.entries {
            if bottom <= target && target < bottom + probability {
                return state;
            }
            bottom += probability;
        }
        self.default
    }
}

#[derive(Debug)]
struct YungPillar {
    target: BlockKey,
    output: Randomizer,
    pillar: Randomizer,
}

impl StructureProcessorType for YungPillarType {
    fn decode(
        &self,
        cx: &dyn ProcessorDecodeContext,
        config: &Tag,
    ) -> Result<Arc<dyn StructureProcessor>, StructureError> {
        let target = field(config, "target_block")?;
        Ok(Arc::new(YungPillar {
            target: block_of(&self.0, field(target, "id")?, "target_block id")?,
            output: Randomizer::decode(cx, &self.0, field(config, "target_block_output")?)?,
            pillar: Randomizer::decode(cx, &self.0, field(config, "pillar_states")?)?,
        }))
    }
}

impl StructureProcessor for YungPillar {
    fn finalize_processing(
        &self,
        cx: &mut dyn ProcessorFinalizeContext,
        _original: &[StructureBlockInfo],
        mut processed: Vec<StructureBlockInfo>,
    ) -> Result<Vec<StructureBlockInfo>, StructureError> {
        let (Some(min_y), Some(max_y)) = (cx.min_y(), cx.max_y()) else {
            return Ok(processed);
        };
        let bounds = cx.settings().bounding_box;
        for info in &mut processed {
            if info.state.block() != self.target {
                continue;
            }
            let [x, y, z] = info.pos;
            // As in the original, only the chunk being generated is written.
            if bounds.is_some_and(|b| x < b.min[0] || x > b.max[0] || z < b.min[2] || z > b.max[2])
            {
                continue;
            }
            let mut random = LegacyRandomSource::new(cx.random_at(info.pos).next_long());
            info.state = self.output.get(&mut random);
            let mut at = [x, y - 1, z];
            while at[1] > min_y && at[1] <= max_y {
                let open = match cx.block(at) {
                    Some(block) => cx.is_air(block) || cx.is_water(at) == Some(true),
                    None => false,
                };
                if !open {
                    break;
                }
                cx.set_block(at, self.pillar.get(&mut random), BlockUpdateFlags::CLIENTS);
                at[1] -= 1;
            }
        }
        Ok(processed)
    }
}
