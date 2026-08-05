#!/bin/bash
rm /tmp/screenshot*.png
cd /home/dave/sync/test/src/rust/qwen && cargo run --release -- --model=/home/dave/sync/test/src/rust/qwen/assets/Apollo\ Soyuz.glb --limit=500
./visual_test.sh

