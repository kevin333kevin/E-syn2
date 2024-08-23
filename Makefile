.PHONY: all compile

all: compile

compile:
	cd e-rewriter/ && CARGO_BUILD_JOBS=$(shell nproc) cargo build --release
	cd graph2eqn/ && CARGO_BUILD_JOBS=$(shell nproc) cargo build --release
	cd e-rewriter/src/egraph-serialize/ && CARGO_BUILD_JOBS=$(shell nproc) cargo build --release 
	cd abc/ && make -j64
	cd process_json/ && CARGO_BUILD_JOBS=$(shell nproc) cargo build --release
	cd extraction-gym/ && CARGO_BUILD_JOBS=$(shell nproc) cargo build --release 