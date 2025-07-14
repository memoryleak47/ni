class Int:
    def __init__(self, i):
        print("building an A!")
        self.i = i

    def __add__(self, other):
        return Int(self.i + other.i)

class DerivedInt(Int):
    pass

d1 = DerivedInt(2)
d2 = DerivedInt(5)
print((d1 + d2).i)
