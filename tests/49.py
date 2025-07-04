def foo():
    class A(BaseException):
        pass

    try:
        raise A("ok")
    except A:
        print("nice")

foo()
