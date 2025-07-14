class A:
    def a(self):
        print("a")

class B(A):
    def b(self):
        print("b")

class C(B):
    def c(self):
        print("c")

class D(C):
    def d(self):
        print("d")

d = D()
d.a()
d.b()
d.c()
d.d()
