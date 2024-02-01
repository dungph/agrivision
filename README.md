# Agrivision

prerequired: `rust-lang/cargo`

build: `cargo build --release`

run: `cargo run -- --help`

```
cargo r --release -- \
    --model-path ~/50nround.safetensors \
    --model-size n \
    --num-classes 3 \
    --min-acc 0.2 \
    --min-nms 0.45 \
    --snapshot-url http://joy3:8080/photo.jpg \
    --stream-url http://joy3:8080/video
```
