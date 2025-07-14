r = range(0, 3)
i = r.__iter__()
print(i.__next__())
print(i.__next__())
print(i.__next__())
try:
    print(i.__next__())
except:
    print("iteration stopped!")
