class A:
    x = "irrelevant1"
    a = "irrelevant2"
    def __init__(self, x):
        print(x)
        self.a = x
        print("constructor called!")

a = A(5)
print(a.a)
