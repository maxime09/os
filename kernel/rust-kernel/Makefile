.PHONY: all
all:
	cargo build --target x86_64-myos.json
	cp target/x86_64-myos/debug/librust_kernel.a .

clean:
	cargo clean
	rm -f librust_kernel.a