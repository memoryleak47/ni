def foo():
	global x
	print(x)
	x = x+1
	print(x)

x = 1
foo()
print(x)
