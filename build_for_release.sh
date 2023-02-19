#!/bin/sh

# Using musl means everything is compiled statically
# which results in an independant binary be able to
# run in on any x86_64 based system. (That's how I understand it)
read -p "Version Number (e.g. v1.2.3): " versionNumber
cmds[0]="git tag -a $versionNumber -m $versionNumber"
cmds[1]="cargo build -r --target x86_64-unknown-linux-musl"
cmds[2]="cp target/x86_64-unknown-linux-musl/release/songs ."
cmds[3]="mv songs songs_$versionNumber_x86_64-unknown-linux-musl"
