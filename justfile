
watch-sass:
    node-sass --watch scss -o static/css 

watch-source:
    cargo watch -x "run"
clean:
  cargo clean
