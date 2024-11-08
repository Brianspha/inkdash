
test:
	@cargo test
build: 
	@cargo build

lint :
	@cargo contract build --lint 

build_pop:
	@pop build

test_pop:
	@pop test contract

start_local_chain:
	@pop up parachain -f ./pop.toml -y

