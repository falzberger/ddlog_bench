GENERATOR=generator.py
UPDATES=updates
PYTHON=python3

all: generate compile execute-chain execute-single execute-multiple

compile:
	echo "Compiling query.dl to query_ddlog..."
	ddlog --nested-ts-32 -i query.dl
	(cd query_ddlog && cargo build --release)
	cd ..

generate:
	mkdir -p data
	echo "Generating chain.csv (1 million nodes) ... "
	$(PYTHON) $(GENERATOR) --trees 1 --degree 1 --depth 1000000 data/chain.csv
	echo "Generating single.csv (approx. 500k nodes) ... "
	$(PYTHON) $(GENERATOR) --trees 1 --degree 5 --depth 9 data/single.csv
	echo "Generating multiple.csv (approx. 150k nodes) ..."
	$(PYTHON) $(GENERATOR) --trees 1000 --degree 5 --depth 4 data/multiple.csv

execute-chain:
	cargo run --release -- -i Edge data/chain.csv \
	-u $(UPDATES)/chain_remove_root.csv \
	-u $(UPDATES)/chain_insert_root.csv \
	-u $(UPDATES)/chain_split_half.csv \
	-u $(UPDATES)/chain_combine_half.csv \
	-u $(UPDATES)/chain_split_quarter.csv \
	-u $(UPDATES)/chain_combine_quarter.csv

execute-single:
	cargo run --release -- -i Edge data/single.csv \
	-u $(UPDATES)/single_remove_root.csv \
	-u $(UPDATES)/single_insert_root.csv

execute-multiple:
	cargo run --release -- -i Edge data/multiple.csv \
	-u $(UPDATES)/multiple_insert_root.csv \
	-u $(UPDATES)/multiple_remove_root.csv

clean:
	rm -rf data *.out

.PHONY: all compile execute-chain execute-single execute-multiple clean
