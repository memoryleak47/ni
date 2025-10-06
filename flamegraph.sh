#!/bin/bash

RUSTFLAGS="-C force-frame-pointers=yes" cargo flamegraph -- atests/12.safe.py --analyze
