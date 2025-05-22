class A:
    x = "irrelevant"
    a = "irrelevant"
    def __init__(self, x):
        self.a = x
        print("constructor called!")

a = A(5)
print(a.a)
