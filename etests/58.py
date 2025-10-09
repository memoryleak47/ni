i = 4
i += 4
print(i)
i -= 4
print(i)

class A:
    def __mul__(self, x):
        print("wow")
        self.x = 42
        return self

a = A()
a *= 3
print(a.x)
