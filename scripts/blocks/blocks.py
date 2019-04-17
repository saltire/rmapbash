import csv
import os

from PIL import Image


blocktexdir = '[minecraft data dir]/1.13.2/assets/minecraft/textures/block/'

colors = {}
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

    colors[imgfile[:-4]] = color

with open('blocknames.csv', 'r') as blockfile:
    blocks = [b.strip() for b in blockfile.readlines()]

    with open('blocks.csv', 'w') as csvfile:
        writer = csv.writer(csvfile)

        writer.writerow(['block', 'r', 'g', 'b', 'a'])
        for block in blocks:
            writer.writerow([block, *colors.get(block, ['','','',''])])


with open('texturecolors.csv', 'w') as csvfile:
    writer = csv.writer(csvfile)

    for texture, color in colors.items():
        writer.writerow([texture, *color])
