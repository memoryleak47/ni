#!/bin/bash

RUSTFLAGS="-C force-frame-pointers=yes" cargo flamegraph -- atests/11.safe.py --analyze
