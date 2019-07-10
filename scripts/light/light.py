import csv
import os

from PIL import Image


currentdir = os.path.dirname(__file__)

for mode in ['day', 'night', 'nether', 'end']:
    img = Image.open(os.path.join(currentdir, '{}.png'.format(mode)))
    print(mode, img)
    pix = img.load()

    with open(os.path.join(currentdir, '../../resources/light/{}.csv'.format(mode)), 'w') as csvfile:
        writer = csv.writer(csvfile)
        writer.writerow(['sky', 'block', 'r', 'g', 'b'])

        for sky in range(16):
            for block in range(16):
                r, g, b = pix[block, sky]
                writer.writerow([sky, block, r, g, b])
