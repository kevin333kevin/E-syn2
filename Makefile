.PHONY: all compile multi_round dag_cost

all: multi_round dag_cost compile

compile:
	cd e-rewriter/ && CARGO_BUILD_JOBS=$(shell nproc) cargo build --release
	cd graph2eqn/ && CARGO_BUILD_JOBS=$(shell nproc) cargo build --release
	cd abc/ && make -j64
	cd process_json/ && CARGO_BUILD_JOBS=$(shell nproc) cargo build --release
	cd extraction-gym/ && CARGO_BUILD_JOBS=$(shell nproc) cargo build --release

multi_round:
	@if [ ! -f e-rewriter/target/release/e-rewriter-multi_round ]; then \
		cd e-rewriter/ && CARGO_BUILD_JOBS=$(shell nproc) cargo build --release --features multi_round; \
		mv target/release/e-rewriter target/release/e-rewriter-multi_round; \
	else \
		echo "Binary for e-rewriter-multi_round already exists."; \
	fi

dag_cost:
	@if [ ! -f e-rewriter/target/release/e-rewriter-dag_cost ]; then \
		cd e-rewriter/ && CARGO_BUILD_JOBS=$(shell nproc) cargo build --release --features dag_cost; \
		mv target/release/e-rewriter target/release/e-rewriter-dag_cost; \
	else \
		echo "Binary for e-rewriter-dag_cost already exists."; \
	fi