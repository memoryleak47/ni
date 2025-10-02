class A:
    def __eq__(self, other):
        print("eq!")
        return True

    def __ne__(self, other):
        print("ne!")
        return True

print(1 == 1)
print(1 != 1)
print(2 == 1)
print(2 != 1)
print(type(2) == type(1))
print(type(2) != type(1))
print(range(5) == range(4))
print(range(5) != range(4))
print(A() == A())
print(A() != A())

