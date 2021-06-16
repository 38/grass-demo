#!/bin/bash

die() {
	echo $1
	exit 1
}

which jupyter || die "Unable to find jupyter from \$PATH, please install jupyter first"
which cargo || die "Unable to find Rusot toolchain, go to http://rustup.rs for installation instructions"

which evcxr_jupyter || cargo install evcxr_jupyter

evcxr_jupyter --install 

exec jupyter notebook --ip=0.0.0.0 --no-browser
