from PIL import Image as i
from PIL import ImageEnhance as enhance
import maplegend

def main():
    pass

def sharpenimage(image):
    
    enhance.Sharpness(image).enhance(2.0)
    i.show(image)




if __name__ == '__main__':

    image = i.open('rawmap.png')
    i.show(image)

    sharpenimage(image)
    i.show(image)

    main()
