RUSTC ?= rustc

dummy1 := $(shell mkdir bin 2> /dev/null)

all: src/main.rs
		$(RUSTC) -L libs/ -o bin/cog src/main.rs 

clean:
	rm -rf bin/*

