l = [1, 2, 3][::1]
print(l.__len__())
for i in range(3):
    print(l[i])

l = [1, 2, 3][::-1]
print(l.__len__())
for i in range(3):
    print(l[i])
