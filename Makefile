RUSTC ?= rustc

dummy1 := $(shell mkdir bin 2> /dev/null)

all: src/main.rs
		$(RUSTC) -o bin/rusty-jack src/main.rs 

clean:
	rm -rf bin/*

