class A:
    def __init__(self):
        self.name = input()

a = A()
b = A()

l = [a, b]
l[0].name = "wow"

assert(a.name == "wow")
