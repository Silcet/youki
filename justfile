alias build := youki-release
alias youki := youki-dev

ROOT := `git rev-parse --show-toplevel`


# build

# build all binaries
build-all: youki-release rust-oci-tests-bin runtimetest

# build youki in dev mode
youki-dev:
    ./scripts/build.sh -o {{ROOT}} -c youki

# build youki in release mode
youki-release:
    ./scripts/build.sh -o {{ROOT}} -r -c youki

# build runtimetest binary
runtimetest:
    ./scripts/build.sh -o {{ROOT}} -r -c runtimetest

# build rust oci tests binary
rust-oci-tests-bin:
    ./scripts/build.sh -o {{ROOT}} -r -c integration-test



# Tests


# run oci tests
test-oci: oci-tests rust-oci-tests

# run all tests except rust-oci 
test-all: unittest featuretest oci-tests containerd-test # currently not doing rust-oci here

# run cargo unittests
unittest:
    cd ./crates
    LD_LIBRARY_PATH=${HOME}/.wasmedge/lib cargo test --all --all-targets --all-features

# run purmutated faeture compilation tests
featuretest:
    ./scripts/features_test.sh

# run oci integration tests
oci-tests: youki-release
    ./scripts/oci_integration_tests.sh {{ROOT}}


# run rust oci integration tests
rust-oci-tests: youki-release runtimetest rust-oci-tests-bin
    ./scripts/rust_integration_tests.sh {{ROOT}}/youki

# validate rust oci integration tests on runc
validate-rust-oci-runc: runtimetest rust-oci-tests-bin
    ./scripts/rust_integration_tests.sh runc

# run containerd integration tests
containerd-test: youki-dev
	VAGRANT_VAGRANTFILE=Vagrantfile.containerd2youki vagrant up
	VAGRANT_VAGRANTFILE=Vagrantfile.containerd2youki vagrant provision --provision-with test

#misc

# run bpftrace hack
hack-bpftrace:
    BPFTRACE_STRLEN=120 ./hack/debug.bt

# run linting on project
lint:
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings

# cleans up generated artifacts
clean:
    ./scripts/clean.sh {{ROOT}}