try:
    raise BaseException("ok")
except:
    print("nice")


try:
    raise BaseException("ok")
except BaseException:
    print("nice")
