# roguelike

This is currently hosted at https://agoodlet.com/wasm/

## setup (local):
Nothing really required here, just run the following <br>
- cargo run _this will compile and run an instance of this on your local machine_

## setup (hosted):
You will need to make sure that you have wasm added as a target for rust - <br>
`rustup target add wasm32-unknown-unknown` <br>
and you will need wasm-bindgen-cli - <br>
`cargo install wasm-bindgen-cli` <br>
then you can follow the instructions here to build - <br>
- cargo build --release --target wasm32-unknown-unknown
- wasm-bindgen ./target/wasm32-unknown-unknown/release/rogulike.wasm --out-dir wasm --no-modules --no-typescript
- copy the contents of ./wasm to you websites root directory
