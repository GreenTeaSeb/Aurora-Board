default: build

run: build
  RUST_LOG=info cargo run

build: build-frontend build-backend
build-frontend: _format-frontend
  sassc scss/style.scss css/style.css
  sassc scss/image.scss css/image.css
build-backend: _format-backend
  cargo build
  
_format-backend: 
  cargo fmt

_format-frontend:
  scssfmt scss/style.scss scss/style.scss
  scssfmt scss/image.scss scss/image.scss
  
clean:
  cargo clean
  rm css/style.css css/image.css
