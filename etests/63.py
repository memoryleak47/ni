l = list(range(100))[1:21:4]
print(l.__len__())
for i in range(l.__len__()):
    print(l[i])

print("----")
l[0:5:2] = [222, 333, 444]

print(l.__len__())
for i in range(l.__len__()):
    print(l[i])
