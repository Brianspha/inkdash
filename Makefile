
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

test_pop_e2e:
	@pop test contract --e2e 

start_local_chain:
	@pop up parachain -f ./pop.toml -y

build_workspace:
	@cargo contract build --manifest-path ./contracts/game/Cargo.toml
	@cargo contract build --manifest-path ./contracts/erc20/Cargo.toml
	@cargo contract build