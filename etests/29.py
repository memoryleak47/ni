class A:
    def foo(self):
        self.x = 3
        print(self.x)

a = A()
f = a.foo
f()
