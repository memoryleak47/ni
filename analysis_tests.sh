#!/bin/sh

[ ! -d .git ] && "You need to be in the repo root to execute this" && exit

function run_case() {
    echo "========="
    echo "atests/$f"
    echo "========="
    echo

    res1=$(RUSTFLAGS=-Awarnings timeout -v 20s cargo r --release -q "atests/$f" --analyze)
    if [[ ! "$?" == 0 ]]; then
        echo error!
        echo $res1
    fi
    res2=$(echo $f | cut -d '.' -f 2)
    if [[ ! "$res1" == "$res2" ]]; then
        echo different output:
        echo "-- analysis output: $res1"
        echo "-- ground truth: $res2"
    fi

    echo
    echo
}

if [ -z "$1" ]; then
    for f in $(find atests -type f | cut -d "/" -f 2 | sort -h)
    do
        run_case "$f"
    done
else
    f=$(cd atests; ls $1.*)
    run_case "$f"
fi
