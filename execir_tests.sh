#!/bin/sh

[ ! -d .git ] && "You need to be in the repo root to execute this" && exit

for i in $(find etests -type f -name "*.py" | cut -d "/" -f 2 | cut -d "." -f 1 | sort -h)
do
    inputfile=etests/${i}.txt
    if [ ! -e $inputfile ]; then
        inputfile=/dev/null
    fi
    echo "========="
    echo "etests/${i}.py"
    echo "========="
    echo

    res1=$(RUSTFLAGS=-Awarnings timeout 3s cargo r --release -q "etests/${i}.py" < "$inputfile")
    if [[ ! "$?" == 0 ]]; then
        echo error!
        echo $res1
        exit
    fi
    echo "$res1"
    res2=$(python "etests/${i}.py" < "$inputfile")
    if [[ ! "$res1" == "$res2" ]]; then
        echo different output:
        echo python:
        echo "$res2"
        exit
    fi

    echo
    echo
done
