class LinkedList:
    def __init__(self, next, val):
        self.next = next
        self.val = val

l = None
i = 0
while input() == "":
    l = LinkedList(l, i)
    i = i+1

v = 42
while (l == None) == False:
    v = l.val
    l = l.next

b = type(v) == type(35)
print(b)
assert(b)
