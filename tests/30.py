class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y

    def __add__(self, other):
        return Point(self.x + other.x, self.y + other.y)

    def dump(self):
        print(self.x)
        print(self.y)

a = Point(1, 1)
b = Point(2, 3)
c = a+b
c.dump()
