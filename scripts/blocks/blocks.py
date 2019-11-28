import csv
import os

from PIL import Image

from shapes.shapes import get_shapes
from textures.textures import get_texturecolors


currentdir = os.path.dirname(__file__)

# Get texture/shape data.
texturecolors = get_texturecolors()
# texturecolors = {}
# with open(os.path.join(currentdir, 'textures/texturecolors.csv'), 'r') as csvfile:
#     for line in csvfile.readlines():
#         texture, r, g, b, a = line.strip().split(',')
#         texturecolors[texture] = r, g, b, a

shapes = get_shapes()
# shapes = {}
# with open(os.path.join(currentdir, 'shapes/shapes.csv'), 'r') as csvfile:
#     for line in csvfile.readlines():
#         shapename, shape = line.strip().split(',')
#         shapes[shapename] = shape

# Get a list of all blocks.
with open(os.path.join(currentdir, 'blocknames.csv'), 'r') as csvfile:
    blocknames = [b.strip() for b in csvfile.readlines()]

# Get color data for each block.
blocks = {}

with open(os.path.join(currentdir, 'blockcolors.csv'), 'r') as csvfile:
    for line in csvfile.readlines():
        block, r1, g1, b1, a1, r2, g2, b2, a2 = line.strip().split(',')
        if (r1, g1, b1, a1) != ('', '', '', ''):
            blocks.setdefault(block, {})['color1'] = {'color': (r1, g1, b1, a1)}
        if (r2, g2, b2, a2) != ('', '', '', ''):
            blocks.setdefault(block, {})['color2'] = {'color': (r2, g2, b2, a2)}

with open(os.path.join(currentdir, 'copyblockcolor.csv'), 'r') as csvfile:
    for line in csvfile.readlines():
        block, blocktocopy1, blocktocopy2 = line.strip().split(',')
        if blocktocopy1 != '':
            blocks.setdefault(block, {})['color1'] = {'copyblock': blocktocopy1}
        if blocktocopy2 != '':
            blocks.setdefault(block, {})['color2'] = {'copyblock': blocktocopy2}

with open(os.path.join(currentdir, 'copytexturecolor.csv'), 'r') as csvfile:
    for line in csvfile.readlines():
        block, texture1, texture2 = line.strip().split(',')
        if texture1 != '':
            blocks.setdefault(block, {})['color1'] = {'copytexture': texture1}
        if texture2 != '':
            blocks.setdefault(block, {})['color2'] = {'copytexture': texture2}

with open(os.path.join(currentdir, 'blockbiomes.csv'), 'r') as csvfile:
    for line in csvfile.readlines():
        block, biome = line.strip().split(',')
        blocks.setdefault(block, {})['biome'] = biome

with open(os.path.join(currentdir, 'blockshapes.csv'), 'r') as csvfile:
    for line in csvfile.readlines():
        block, state, shapename = line.strip().split(',')
        (blocks.setdefault(block, {}).setdefault('shape', {}).setdefault('states', {})
            [state]) = shapes[shapename]

with open(os.path.join(currentdir, 'copyblockshapes.csv'), 'r') as csvfile:
    for line in csvfile.readlines():
        block, blocktocopy = line.strip().split(',')
        blocks.setdefault(block, {})['shape'] = {'copyblock': blocktocopy}

with open(os.path.join(currentdir, 'blocklogged.csv'), 'r') as csvfile:
    for line in csvfile.readlines():
        block, state = line.strip().split(',')
        blocks.setdefault(block, {}).setdefault('shape', {})['logstate'] = state

def get_block_color(block, key):
    colordata = blocks.get(block, {}).get(key, {})

    if 'color' in colordata:
        return colordata['color']
    if 'copytexture' in colordata:
        return texturecolors[colordata['copytexture']]
    if 'copyblock' in colordata:
        # Copy the block's primary color (for now).
        return get_block_color(colordata['copyblock'], 'color1')

def add_state(state_string, new_state):
    return '&'.join([s for s in [state_string, new_state] if s])

def get_block_shapes(block):
    shapedata = blocks.get(block, {}).get('shape', {})

    if 'copyblock' in shapedata:
        stateshapes = get_block_shapes(shapedata['copyblock'])
    else:
        stateshapes = shapedata.get('states', {'': shapes['solid shadows']}).copy()

    # Add waterlogged shapes if applicable.
    if 'logstate' in shapedata:
        if shapedata['logstate']:
            # If the block has a specific waterlogged state variable,
            # add it to a copy of each of the existing states.
            stateshapes.update({add_state(state, shapedata['logstate']): shape.replace('0', '7')
                                for state, shape in stateshapes.items()})
        else:
            # Otherwise the block is always waterlogged, so replace all of the existing states.
            stateshapes = {state: shape.replace('0', '7') for state, shape in stateshapes.items()}

    return stateshapes


# Compile final blocks.csv from all read data.
with open(os.path.join(currentdir, '../../resources/blocks.csv'), 'w') as csvfile:
    writer = csv.writer(csvfile)

    writer.writerow(['name', 'r', 'g', 'b', 'a', 'r2', 'g2', 'b2', 'a2', 'biome', 'state', 'shape'])
    writer.writerow(['', '', '', '', '', '', '', '', '', '', '', ''])

    for block in blocknames:
        if block not in blocks:
            print('No texture for', block)

        color1 = get_block_color(block, 'color1')
        color2 = get_block_color(block, 'color2')
        biome = blocks.get(block, {}).get('biome', '')
        stateshapes = get_block_shapes(block)

        for state, shape in stateshapes.items():
            writer.writerow([
                block,
                *(color1 or ('', '', '', '')),
                *(color2 or ('', '', '', '')),
                biome,
                state,
                shape,
            ])
