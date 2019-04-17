import csv
import os

from PIL import Image


foliage = Image.open('./foliage.png')
grass = Image.open('./grass.png')
fpix = foliage.load()
gpix = foliage.load()

biomes = []

with open('./biomes.csv', 'w') as outfile:
    writer = csv.writer(outfile)
    writer.writerow(['id', 'name', 'fr', 'fg', 'fb', 'gr', 'gg', 'gb'])

    with open('./biomevalues.csv', 'r') as csvfile:
        for id, name, temp, rain in csv.reader(csvfile):
            adjtemp = max(0, min(1, float(temp)))
            adjrain = max(0, min(1, float(rain))) * adjtemp
            x = int(255 * (1 - adjtemp))
            y = int(255 * (1 - adjrain))
            fr, fg, fb = fpix[x, y][:3]
            gr, gg, gb = gpix[x, y][:3]

            writer.writerow([id, name, fr, fg, fb, gr, gg, gb])
