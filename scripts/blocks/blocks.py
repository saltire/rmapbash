import csv
import os

from PIL import Image


datadir = '[path to contents of extracted minecraft version .jar file]'
blocktexdir = datadir + '/assets/minecraft/textures/block/'

currentdir = os.path.dirname(__file__)


# Read blocknames, blockcolors, copyblock, copytexture, blockbiomes, texturecolors from csv.

with open(os.path.join(currentdir, 'blocknames.csv'), 'r') as csvfile:
    blocknames = [b.strip() for b in csvfile.readlines()]

blockcolors = {}
blockcolors2 = {}
with open(os.path.join(currentdir, 'blockcolors.csv'), 'r') as csvfile:
    for line in csvfile.readlines():
        block, r, g, b, a, r2, g2, b2, a2 = line.strip().split(',')
        blockcolors[block] = r, g, b, a
        if (r2, g2, b2, a2) != ('', '', '', ''):
            blockcolors2[block] = r2, g2, b2, a2

copyblock = {}
copyblock2 = {}
with open(os.path.join(currentdir, 'copyblock.csv'), 'r') as csvfile:
    for line in csvfile.readlines():
        block, blocktocopy, blocktocopy2 = line.strip().split(',')
        if blocktocopy != '':
            copyblock[block] = blocktocopy
        if blocktocopy2 != '':
            copyblock2[block] = blocktocopy2

copytexture = {}
copytexture2 = {}
with open(os.path.join(currentdir, 'copytexture.csv'), 'r') as csvfile:
    for line in csvfile.readlines():
        block, texture, texture2 = line.strip().split(',')
        if texture != '':
            copytexture[block] = texture
        if texture2 != '':
            copytexture2[block] = texture2

biomes = {}
with open(os.path.join(currentdir, 'blockbiomes.csv'), 'r') as csvfile:
    for line in csvfile.readlines():
        block, biome = line.strip().split(',')
        biomes[block] = biome

texturecolors = {}
with open(os.path.join(currentdir, 'textures/texturecolors.csv'), 'r') as csvfile:
    for line in csvfile.readlines():
        texture, r, g, b, a = line.strip().split(',')
        texturecolors[texture] = r, g, b, a


# Compile final blocks.csv from all read data.

with open(os.path.join(currentdir, '../../resources/blocks.csv'), 'w') as csvfile:
    writer = csv.writer(csvfile)

    writer.writerow(['name', 'r', 'g', 'b', 'a', 'r2', 'g2', 'b2', 'a2', 'biome'])
    writer.writerow(['', '', '', '', '', '', '', '', '', ''])

    for block in blocknames:
        color = None
        color2 = None

        if block in blockcolors:
            color = blockcolors[block]
        elif block in copytexture:
            texture = copytexture[block]
            color = texturecolors[texture]
        elif block in copyblock:
            blocktocopy = copyblock[block]
            if blocktocopy in blockcolors:
                color = blockcolors[blocktocopy]
            elif blocktocopy in copytexture:
                texture = copytexture[blocktocopy]
                color = texturecolors[texture]

        if block in blockcolors2:
            color2 = blockcolors2[block]
        elif block in copytexture2:
            texture = copytexture2[block]
            color2 = texturecolors[texture]
        elif block in copyblock2:
            blocktocopy = copyblock2[block]
            # Copy from block's primary color.
            if blocktocopy in blockcolors:
                color2 = blockcolors[blocktocopy]
            elif blocktocopy in copytexture:
                # Copy from block's primary texture color.
                texture = copytexture[blocktocopy]
                color2 = texturecolors[texture]

        if color is None:
            print('No texture for', block)

        writer.writerow([
            block,
            *(color or ('', '', '', '')),
            *(color2 or ('', '', '', '')),
            biomes.get(block, ''),
        ])
