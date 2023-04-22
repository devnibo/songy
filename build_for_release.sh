#!/bin/bash

# Using musl means everything is compiled statically
# which results in an independant binary be able to
# run in on any x86_64 based system. (That's how I understand it)
read -p "Version Number (e.g. v1.2.3): " versionNumber
declare -a cmds=(
	"git tag $versionNumber"
	"cargo build -r --target x86_64-unknown-linux-musl"
	"cp target/x86_64-unknown-linux-musl/release/songs ."
	"mv songs songs_v"$versionNumber"_x86_64-unknown-linux-musl"
)
for (( i=0; i<${#cmds[@]}; i++ ));
do
	echo "${cmds[$i]}"
	bash -c "${cmds[$i]}"
done
