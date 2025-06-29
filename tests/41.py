def foo():
    print("doing things ...")
    raise "ohno"

try:
    foo()
    print("unreachable")
except:
    print("problem happened")
print("post")
