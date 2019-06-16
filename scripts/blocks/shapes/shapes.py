import csv
import os

from PIL import Image


currentdir = os.path.dirname(__file__)

def get_shapes():
    '''Read shapenames from csv and shapes from image.'''
    with open(os.path.join(currentdir, 'shapenames.csv'), 'r') as csvfile:
        shapenames = [l.strip() for l in csvfile.readlines()]

    shapes = {}
    simg = Image.open(os.path.join(currentdir, 'shapes.png'))
    spix = simg.load()
    scolors = [
        (255, 255, 255), # white
        (255, 0, 0),     # red
        (255, 128, 128), # red light
        (128, 0, 0),     # red dark
        (0, 0, 255),     # blue
        (128, 128, 255), # blue light
        (0, 0, 128),     # blue dark
    ]
    s = 0
    for y in range(1, simg.height, 5):
        for x in range(1, simg.width, 5):
            if spix[x, y] == (0, 0, 0):
                continue

            shape = ''
            for sy in range(4):
                for sx in range(4):
                    shape += str(scolors.index(spix[x + sx, y + sy]))

            shapes[shapenames[s]] = shape
            s += 1

    return shapes


if __name__ == '__main__':
    shapes = get_shapes()

    with open(os.path.join(currentdir, 'shapes.csv'), 'w') as csvfile:
        writer = csv.writer(csvfile)

        for shape, pixmap in shapes.items():
            writer.writerow([shape, pixmap])
