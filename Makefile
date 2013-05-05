RUSTC ?= rustc

all: src/main.rs
		$(RUSTC) -o bin/rusty-jack src/main.rs 

clean:
	rm -rf bin/*

