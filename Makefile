release:
	cargo fmt ; cargo build --release

debug:
	cargo fmt ; RUST_LOG="debug" cargo run

graph:
	gprof2dot --format=callgrind callgrind.out -o out.dot
	dot -Tpng out.dot -o target/graph.png

	rm callgrind.out
	rm out.dot
