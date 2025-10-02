class A:
    def __neg__(self):
        return self

a = A()
print(a == --a)

print(1 == --1)
