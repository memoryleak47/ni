print("pre")
try:
    print("within")
    raise "ok"
    print("wrong post")
except:
    print("nice")
print("good post")
