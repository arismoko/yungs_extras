"""Writes YUNG's Extras' features as data: template features, placed features with the
original's placement checks as block predicates, and the biome modifiers.

The original places each piece from Java (AbstractNbtFeature.createTemplateFromCenter): walk
down from the heightmap position P to the surface (P-1), check the ground, then place the
template centred on the origin (corner = origin - size/2 in x and z, no rotation). Here the
template feature does the centring, the checks are block_predicate_filter steps relative to P,
and a final offset moves P to the original's origin.
"""
import json, os, shutil, sys

# The original's data folder: Common/src/main/resources/data/yungsextras in YUNG-GANG/YUNGs-Extras.
UP = sys.argv[1]
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'data', 'yungsextras')
AIR = ['minecraft:air', 'minecraft:cave_air', 'minecraft:void_air']


def st(i, **p):
    return {'id': i, 'properties': p} if p else {'id': i}


def not_air(x, y, z):
    return {'type': 'minecraft:not', 'predicate': {'type': 'minecraft:matching_blocks', 'blocks': AIR, 'offset': [x, y, z]}}


def solid(x, y, z):
    return {'type': 'minecraft:solid', 'offset': [x, y, z]}


SAND = {'type': 'minecraft:matching_block_tag', 'tag': 'minecraft:sand', 'offset': [0, -1, 0]}

FACINGS = ['north', 'east', 'south', 'west']
CANDLES = ['candle', 'white_candle', 'gray_candle', 'light_gray_candle', 'brown_candle', 'green_candle', 'purple_candle', 'black_candle']
BRICKS = {'entries': [{'blockState': st('minecraft:mossy_stone_bricks'), 'probability': 0.45},
                      {'blockState': st('minecraft:cracked_stone_bricks'), 'probability': 0.45}],
          'defaultBlockState': st('minecraft:stone_bricks')}


def candles(lit):
    return [st('minecraft:' + c, candles=str(n), lit=str(lit).lower()) for c in CANDLES for n in range(1, 5)]


# SwampFeatureProcessor. Its passes run on the template's own blocks, so each placeholder is
# rolled once: the brick and stair re-rolls come before the passes that output bricks and stairs.
SWAMP = [
    {'processor_type': 'yungsextras:pick', 'block': 'minecraft:stone_bricks',
     'picks': [{'below': 0.45, 'state': st('minecraft:mossy_stone_bricks')},
               {'below': 0.9, 'state': st('minecraft:cracked_stone_bricks')}],
     'default': {'state': st('minecraft:stone_bricks')}},
    {'processor_type': 'yungsextras:pick', 'block': 'minecraft:stone_brick_stairs',
     'picks': [{'below': 0.6, 'state': st('minecraft:mossy_stone_brick_stairs')}],
     'default': {'state': st('minecraft:stone_brick_stairs')}, 'keep_properties': True},
    # Ruined stairs: a stair of either kind facing a random horizontal way.
    {'processor_type': 'yungsextras:pick', 'block': 'minecraft:light_gray_stained_glass',
     'picks': [{'below': 0.6, 'states': [st('minecraft:mossy_stone_brick_stairs', facing=f) for f in FACINGS]}],
     'default': {'states': [st('minecraft:stone_brick_stairs', facing=f) for f in FACINGS]}},
    # Supports: a brick, with a column of bricks down to the ground.
    {'processor_type': 'yungsextras:yung_pillar', 'target_block': st('minecraft:gray_stained_glass'),
     'target_block_output': BRICKS, 'pillar_states': BRICKS},
    # Candles: one of eight colours, one to four of them, lit one time in ten.
    {'processor_type': 'yungsextras:pick', 'block': 'minecraft:green_candle',
     'picks': [{'below': 0.1, 'states': candles(True)}],
     'default': {'states': candles(False)}},
]

# DesertWellProcessor: brown glass becomes sand or the vanilla well's suspicious sand (at least
# two, each further one a 1-in-10 roll, three at most on average as the original's 2-4);
# yellow glass (wishing wells) becomes sand or suspicious sand with the mod's own loot.
WELL = [
    {'processor_type': 'yungsextras:pick', 'block': 'minecraft:brown_stained_glass', 'picks': [],
     'default': {'state': st('minecraft:suspicious_sand'), 'nbt': {'LootTable': 'minecraft:archaeology/desert_well'}}},
    {'processor_type': 'yungsextras:count_cap', 'blocks': ['minecraft:suspicious_sand'], 'min': 2, 'max': 3,
     'chance': 0.1, 'replacement': st('minecraft:sand')},
    {'processor_type': 'yungsextras:pick', 'block': 'minecraft:yellow_stained_glass',
     'picks': [{'below': 0.4, 'state': st('minecraft:suspicious_sand'), 'nbt': {'LootTable': 'yungsextras:desert/extra_archeology'}}],
     'default': {'state': st('minecraft:sand')}},
]


def corners(x0, z0, dx, dz, y):
    return [(x0, y, z0), (x0, y, z0 + dz), (x0 + dx, y, z0), (x0 + dx, y, z0 + dz)]


def swamp(kind):
    # Not air four below the surface (P-1) at the corner points: y = -5 from P.
    pts = {'arch': [(-2, -5, 0), (2, -5, 0)],
           'double_arch': [(-4, -5, 0), (4, -5, 0)],
           'pillar': corners(-2, -2, 3, 3, -5),
           'cubby': corners(-2, -1, 3, 3, -5),
           'church': corners(-6, -2, 12, 4, -5),
           'ogre': corners(-2, -2, 3, 3, -1)}[kind]
    return [not_air(*p) for p in pts], -1, SWAMP


def desert(kind, radius=0):
    if kind == 'well':
        checks = [SAND]
        for i in range(1, 8):
            y = -1 - i
            r = radius
            checks += [not_air(0, y, 0), not_air(r, y, -r), not_air(r, y, r), not_air(-r, y, r), not_air(-r, y, -r)]
        return checks, -7, WELL
    pts, dy = {'obelisk': (corners(-2, -2, 3, 3, -1), 0),
               'giant_torch': ([(-1, -1, -1), (0, -1, -1), (-1, -1, 0), (0, -1, 0)], 0),
               'chillzone': ([(-1, -1, 0), (1, -1, 1), (0, -1, -1), (1, -1, 0)], 0),
               'ruins_0': (corners(-2, -2, 3, 3, -1), -1)}[kind]
    return [SAND] + [solid(*p) for p in pts], dy, []


SWAMP_TYPES = {'swamp_arch': 'arch', 'swamp_double_arch': 'double_arch', 'swamp_pillar': 'pillar',
               'swamp_cubby': 'cubby', 'swamp_church': 'church', 'swamp_ogre': 'ogre'}
FIXED = {'desert_giant_torch': 'yungsextras:desert/misc/giant_torch', 'desert_chillzone': 'yungsextras:desert/misc/chillzone',
         'desert_ruins_0': 'yungsextras:desert/misc/ruins_0'}


def dump(path, value):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'w') as f:
        json.dump(value, f, indent=2)
        f.write('\n')


def main():
    for sub in ('worldgen/feature', 'worldgen/placed_feature', 'neoforge'):
        shutil.rmtree(os.path.join(OUT, sub), ignore_errors=True)
    base = os.path.join(UP, 'worldgen/configured_feature')
    for root, _, files in os.walk(base):
        for name in sorted(files):
            rel = os.path.relpath(os.path.join(root, name), base)
            cf = json.load(open(os.path.join(root, name)))
            kind = cf['type'].split(':')[1]
            config = cf['config']
            if kind in SWAMP_TYPES:
                checks, dy, procs = swamp(SWAMP_TYPES[kind])
            elif kind == 'desert_well':
                checks, dy, procs = desert('well', config['radius'])
            elif kind == 'desert_obelisk':
                checks, dy, procs = desert('obelisk')
            else:
                checks, dy, procs = desert(kind.removeprefix('desert_'))
            template = config.get('location') or FIXED[kind]
            feature = {'type': 'minecraft:template',
                       'templates': [{'data': {'id': template, 'rotations': ['none']}, 'weight': 1}]}
            if procs:
                feature['processors'] = procs
            dump(os.path.join(OUT, 'worldgen/feature', rel), feature)

            placed = json.load(open(os.path.join(UP, 'worldgen/placed_feature', rel)))
            steps = [s for s in placed['placement'] if s['type'] != 'yungsextras:rng_initializer']
            assert [s['type'] for s in steps] == ['minecraft:rarity_filter', 'minecraft:in_square',
                                                  'minecraft:heightmap', 'minecraft:biome'], rel
            steps.append({'type': 'minecraft:block_predicate_filter',
                          'predicate': {'type': 'minecraft:all_of', 'predicates': checks}})
            if dy:
                steps.append({'type': 'minecraft:offset', 'x': 0, 'y': dy, 'z': 0})
            dump(os.path.join(OUT, 'worldgen/placed_feature', rel), {'feature': placed['feature'], 'placement': steps})
    for name in os.listdir(os.path.join(UP, 'neoforge/biome_modifier')):
        shutil.copy(os.path.join(UP, 'neoforge/biome_modifier', name), os.path.join(os.makedirs(os.path.join(OUT, 'neoforge/biome_modifier'), exist_ok=True) or OUT, 'neoforge/biome_modifier', name))


if __name__ == '__main__':
    main()
