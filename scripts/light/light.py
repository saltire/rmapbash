import csv
import os

from PIL import Image


currentdir = os.path.dirname(__file__)

for mode in ['day', 'night']:
    img = Image.open(os.path.join(currentdir, '{}.png'.format(mode)))
    pix = img.load()

    with open(os.path.join(currentdir, '../../resources/{}.csv'.format(mode)), 'w') as csvfile:
        writer = csv.writer(csvfile)
        writer.writerow(['sky', 'block', 'r', 'g', 'b'])

        for sky in range(16):
            for block in range(16):
                r, g, b = pix[block, sky]
                writer.writerow([sky, block, r, g, b])
