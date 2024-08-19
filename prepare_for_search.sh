#!/bin/sh
# $1 is the folder of chordpro files

shrink() {
	# echo "$1" | tr '[:upper:]' '[:lower:]' | tr -d "\n,%!?:()'\"\`’‘ "
	# 'tr' doesn't handle multi byte characters well
	echo "$1" | awk '{ print tolower($0) }' | sed "s/[(\^$),%!?:()'\"\`’‘ ]//g" | tr -d "\n"
}

for filename in "$1/"*.cho;
do
	name=$(basename "$filename")
	name=${name%.*}
	title=$(shrink "$(cho2txt -t "$filename" | head -n1)")
	content=$(shrink "$(cho2txt "$filename")")
	echo "$name:$title:$content"
done
