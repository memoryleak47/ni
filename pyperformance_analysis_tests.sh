#!/bin/sh

[ ! -d .git ] && "You need to be in the repo root to execute this" && exit

function run_case() {
    echo "========="
    echo "pyperformance_atests/$f"
    echo "========="
    echo

    res1=$(RUSTFLAGS=-Awarnings timeout -v 20s cargo r --release -q "pyperformance_atests/$f" --analyze)
    if [[ ! "$?" == 0 ]]; then
        echo error!
        echo $res1
    fi
    echo "-- analysis output: $res1"

    echo
    echo
}

if [ -z "$1" ]; then
    for f in $(find pyperformance_atests -type f -name "bm_*.py" | cut -d "/" -f 2 | sort -h)
    do
        run_case "$f"
    done
else
    f=$(cd pyperformance_atests; ls bm_*$1*.py)
    run_case "$1"
fi
