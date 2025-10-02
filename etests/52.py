class A:
    def __getitem__(self, a):
        print(type(a) == slice)

a = A()

x = a[3]

x = a[:]
x = a[3:]
x = a[:3]
x = a[3:3]

x = a[::]
x = a[3::]
x = a[:3:]
x = a[::3]
x = a[3:3:]
x = a[:3:3]
x = a[3::3]
x = a[3:3:3]
