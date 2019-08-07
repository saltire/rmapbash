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

        im = Image.open(os.path.join(blocktexdir, imgfile)).convert('RGBA')
        pix = im.load()
        pixels = []
        for x in range(im.width):
            for y in range(im.height):
                if pix[x, y][3] > 0:
                    pixels.append(pix[x, y])

        if len(pixels) == 0:
            color = (0, 0, 0, 0)
        else:
            color = (
                int(sum(p[0] for p in pixels) / len(pixels)),
                int(sum(p[1] for p in pixels) / len(pixels)),
                int(sum(p[2] for p in pixels) / len(pixels)),
                int(sum(p[3] for p in pixels) / len(pixels)),
            )

        texturecolors[imgfile[:-4]] = color

    return texturecolors


if __name__ == '__main__':
    texturecolors = get_texturecolors()

    with open(os.path.join(currentdir, 'texturecolors.csv'), 'w') as csvfile:
        writer = csv.writer(csvfile)

        for texture, color in texturecolors.items():
            writer.writerow([texture, *color])
