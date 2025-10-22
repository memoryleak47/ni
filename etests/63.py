l = list(range(100))[1:71:4]
for i in range(18):
    print(l[i])
l[0:5:2] = [22, 33, 44]
for i in range(18):
    print(l[i])
