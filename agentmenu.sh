#!/bin/bash
rm /tmp/screenshot*.png
cd /home/dave/sync/test/src/rust/qwen && cargo run --release -- --model=/home/dave/sync/test/src/rust/qwen/assets/Apollo\ Soyuz.glb --limit=5 --lod=low
#./visual_test.sh

