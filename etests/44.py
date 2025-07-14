def foo():
    pass

def foo2():
    pass

class A:
    def method(self):
        pass

class A2:
    pass

print("a" == "b")
print("a" == "a")
print(1 == 1)
print(1 == 2)
print(True == True)
print(True == False)
print(foo == foo)
print(foo == foo2)
print(A == A)
print(A == A2)
a = A()
a2 = A()
print(a.method == a2.method)
print(a == a)
print(a == a2)
print(None == None)
