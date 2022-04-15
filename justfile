default: build

run: build
  RUST_LOG=info cargo run

build: build-frontend build-backend
build-frontend: _format-frontend
build-backend: _format-backend
  cargo build
  
_format-backend: 
  cargo fmt

_format-frontend:
  
clean:
  cargo clean
