import csv
import os

from PIL import Image


datadir = '[path to contents of extracted minecraft version .jar file]'
blocktexdir = datadir + '/assets/minecraft/textures/block/'

currentdir = os.path.dirname(__file__)

def get_texturecolors():
    '''Get average colors from texture images.'''
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

    return texturecolors


if __name__ == '__main__':
    texturecolors = get_texturecolors()

    with open(os.path.join(currentdir, 'texturecolors.csv'), 'w') as csvfile:
        writer = csv.writer(csvfile)

    for texture, color in texturecolors.items():
        writer.writerow([texture, *color])
