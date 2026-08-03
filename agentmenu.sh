#!/bin/bash
cd /home/dave/sync/test/src/rust/qwen && cargo run --release -- --model=/home/dave/sync/test/src/rust/qwen/assets/Apollo\ Soyuz.glb
./visual_test.sh

