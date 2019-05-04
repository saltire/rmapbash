import csv
import os

from PIL import Image


datadir = '[path to contents of extracted minecraft version .jar file]'
blocktexdir = datadir + '/assets/minecraft/textures/block/'

currentdir = os.path.dirname(__file__)

texturecolors = {}
for imgfile in sorted(os.listdir(blocktexdir)):
    if imgfile[-4:] != '.png':
        continue

    im = Image.open(os.path.join(blocktexdir, imgfile))
    pix = im.load()
    rr = []
    gg = []
    bb = []
    aa = []
    for x in range(im.width):
        for y in range(im.height):
            if im.mode == 'RGBA':
                r, g, b, a = pix[x, y]
                if a > 0:
                    rr.append(r)
                    gg.append(g)
                    bb.append(b)
                    aa.append(a)
            elif im.mode == 'RGB':
                r, g, b = pix[x, y]
                rr.append(r)
                gg.append(g)
                bb.append(b)
            elif im.mode == 'LA':
                r, a = pix[x, y]
                rr.append(r)
                aa.append(a)

    if len(rr) == 0:
        color = (0, 0, 0, 0)
    else:
        if im.mode in ['RGB', 'RGBA']:
            color = (
                int(sum(rr) / len(rr)),
                int(sum(gg) / len(gg)),
                int(sum(bb) / len(bb)),
                int(sum(aa) / len(aa)) if im.mode == 'RGBA' else 255,
            )
        elif im.mode == 'LA':
            l = int(sum(rr) / len(rr))
            color = (l, l, l, int(sum(aa) / len(aa)))

    texturecolors[imgfile[:-4]] = color

with open(os.path.join(currentdir, 'texturecolors.csv'), 'w') as csvfile:
    writer = csv.writer(csvfile)

    for texture, color in texturecolors.items():
        writer.writerow([texture, *color])


with open(os.path.join(currentdir, 'blocknames.csv'), 'r') as csvfile:
    blocknames = [b.strip() for b in csvfile.readlines()]

blockcolors = {}
with open(os.path.join(currentdir, 'blockcolors.csv'), 'r') as csvfile:
    for line in csvfile.readlines():
        block, r, g, b, a = line.strip().split(',')
        blockcolors[block] = r, g, b, a

copyblock = {}
with open(os.path.join(currentdir, 'copyblock.csv'), 'r') as csvfile:
    for line in csvfile.readlines():
        block, blocktocopy = line.strip().split(',')
        copyblock[block] = blocktocopy

copytexture = {}
with open(os.path.join(currentdir, 'copytexture.csv'), 'r') as csvfile:
    for line in csvfile.readlines():
        block, texture = line.strip().split(',')
        copytexture[block] = texture

biomes = {}
with open(os.path.join(currentdir, 'blockbiomes.csv'), 'r') as csvfile:
    for line in csvfile.readlines():
        block, biome = line.strip().split(',')
        biomes[block] = biome


with open(os.path.join(currentdir, '../../resources/blocks.csv'), 'w') as csvfile:
    writer = csv.writer(csvfile)

    writer.writerow(['name', 'r', 'g', 'b', 'a', 'biome'])
    writer.writerow(['', '', '', '', '', ''])

    for block in blocknames:
        color = None

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

        if color is None:
            print('No texture for', block)

        writer.writerow([block, *(color or ('', '', '', '')), biomes.get(block, '')])
