import csv
import os

from PIL import Image


currentdir = os.path.dirname(__file__)

foliage = Image.open(os.path.join(currentdir, 'foliage.png'))
grass = Image.open(os.path.join(currentdir, 'grass.png'))
fpix = foliage.load()
gpix = foliage.load()

biomes = []

with open(os.path.join(currentdir, '../../resources/biomes.csv'), 'w') as outfile:
    writer = csv.writer(outfile)
    writer.writerow(['id', 'name', 'fr', 'fg', 'fb', 'gr', 'gg', 'gb', 'wr', 'wg', 'wb'])

    with open(os.path.join(currentdir, 'biomevalues.csv'), 'r') as csvfile:
        for id, name, temp, rain in csv.reader(csvfile):
            adjtemp = max(0, min(1, float(temp)))
            adjrain = max(0, min(1, float(rain))) * adjtemp
            x = int(255 * (1 - adjtemp))
            y = int(255 * (1 - adjrain))
            fr, fg, fb = fpix[x, y][:3]
            gr, gg, gb = gpix[x, y][:3]
            wr, wg, wb = (224, 255, 174) if 'swamp' in name else ('', '', '')

            writer.writerow([id, name, fr, fg, fb, gr, gg, gb, wr, wg, wb])
