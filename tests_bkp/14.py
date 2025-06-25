x = 0
while True:
	if x == 10:
		print("breaking!")
		break

	j = 0

	# nested loop
	while True:
		j = j+1
		if j > 4:
			break
		continue

	x = x+1
	print("continuing!")
	continue
