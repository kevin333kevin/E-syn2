FEATURES ?=
FLAGS=--release --features=$(FEATURES)

EXTRACTORS=$(shell cargo run -q $(FLAGS) -- --extractor=print)

PROGRAM=target/release/extraction-gym

SRC=$(shell find . -name '.rs') Cargo.toml Cargo.lock
DATA=$(shell find data -name '*.json')

TARGETS=

.PHONY: all
all:  nits bench 
#process_out_json

define run-extraction
TARGETS += $(1:data/%=output/%)-$(2).json
$(1:data/%=output/%)-$(2).json: $(1)
	@mkdir -p $$(dir $$@)
	$(PROGRAM) $$< --extractor=$(2) --out=$$@
endef

$(foreach ext,$(EXTRACTORS),\
	$(foreach data,$(DATA),\
        $(eval $(call run-extraction,$(data),$(ext)))\
    )\
)

.PHONY: bench
bench: $(TARGETS)

$(PROGRAM): $(SRC)
	cargo build $(FLAGS)


.PHONY: nits
nits:
	rustup component add rustfmt clippy
	cargo clean --doc


#.PHONY: process_out_json
#process_out_json:
#	python process_out_json.py