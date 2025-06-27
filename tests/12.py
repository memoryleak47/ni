def fibo(x):
	if x < 2:
		return x
	return fibo(x-1) + fibo(x-2)

i = 0
while i < 10:
	print(fibo(i))
	i = i+1
