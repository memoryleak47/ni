class A:
    pass

print(type(1) == type(2))
print(type("a") == type("b"))
print(type(True) == type(False))
print(type([]) == type([]))
print(type(A) == type(A))
print(type(A()) == type(A()))
