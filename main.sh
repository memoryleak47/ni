#!/bin/bash

cargo b --release
for x in $(ls tests/)
do
	cargo r --release tests/$x
done
