#!/bin/sh

[ ! -d .git ] && "You need to be in the repo root to execute this" && exit

for f in $(find atests -type f | cut -d "/" -f 2 | sort -h)
do
    echo "========="
    echo "atests/$f"
    echo "========="
    echo

    res1=$(RUSTFLAGS=-Awarnings timeout 3s cargo r --release -q "atests/$f" --analyze)
    if [[ ! "$?" == 0 ]]; then
        echo error!
        echo $res1
        exit
    fi
    res2=$(echo $f | cut -d '.' -f 2)
    if [[ ! "$res1" == "$res2" ]]; then
        echo different output:
        echo "-- analysis output: $res1"
        echo "-- ground truth: $res2"
        exit
    fi

    echo
    echo
done
