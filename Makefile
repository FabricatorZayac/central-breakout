##
# w4-game
#
# @file
# @version 0.1

.PHONY: run all clean

run: all
	w4 run target/wasm32-unknown-unknown/release/cart.wasm

all:
	cargo build --release

clean:
	cargo clean

# end
