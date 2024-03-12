.PHONY: all compile feature1 feature2

all: feature1 feature2 compile

compile:
	cd e-rewriter/ && CARGO_BUILD_JOBS=$(shell nproc) cargo build --release
	cd graph2eqn/ && CARGO_BUILD_JOBS=$(shell nproc) cargo build --release
	cd abc/ && make -j64
	cd process_json/ && CARGO_BUILD_JOBS=$(shell nproc) cargo build --release

feature1:
	@if [ ! -f e-rewriter/target/release/e-rewriter-feature1 ]; then \
		cd e-rewriter/ && CARGO_BUILD_JOBS=$(shell nproc) cargo build --release --features feature1; \
		mv target/release/e-rewriter target/release/e-rewriter-feature1; \
	else \
		echo "Binary for e-rewriter-feature1 already exists."; \
	fi

feature2:
	@if [ ! -f e-rewriter/target/release/e-rewriter-feature2 ]; then \
		cd e-rewriter/ && CARGO_BUILD_JOBS=$(shell nproc) cargo build --release --features feature2; \
		mv target/release/e-rewriter target/release/e-rewriter-feature2; \
	else \
		echo "Binary for e-rewriter-feature2 already exists."; \
	fi