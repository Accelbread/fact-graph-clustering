workdir:
	mkdir workdir
	mkdir workdir/graphs
	mkdir workdir/input

prepreocess:
	cargo run --release --bin prepreocess

generate:
	cargo run --release --bin generate

cluster:
	cargo run --release --bin cluster
