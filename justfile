default:
    cargo run --release

watch-sass:
    sass --watch scss:static/css --style compressed
watch-src:
    cargo watch -x "run"

sass:
    sass scss:static/css --style compressed

src:
    cargo run

clean:
  cargo clean
