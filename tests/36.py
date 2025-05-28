class Super:
    x = "1"
    y = "2"
    z = "3"

    def foo(self):
        print(self.x)
        print(self.y)
        print(self.z)

class Base(Super):
    x = "4"
    y = "5"
    pass

a = Base()
a.x = "6"


a.foo()

f = a.foo
f()
