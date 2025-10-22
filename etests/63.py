l = list(range(100))[1:21:4]
for i in range(5):
    print(l[i])

print("----")
l[0:5:2] = [222, 333, 444]

for i in range(5):
    print(l[i])
