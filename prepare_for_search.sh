#!/bin/sh
# $1 is the folder of chordpro files

shrink() {
	# echo "$1" | tr '[:upper:]' '[:lower:]' | tr -d "\n,%!?:()'\"\`’‘ "
	# 'tr' doesn't handle multi byte characters well
	echo "$1" | awk '{ print tolower($0) }' | sed "s/[(\^$),%!?:()'\"\`’‘ ]//g" | tr -d "\n"
}

for file in $(find "$1" -name '*.cho')
do
	name=$(basename "$file")
	name=${name%.*}
	title=$(shrink "$(cho2txt -t "$file" | head -n1)")
	content=$(shrink "$(cho2txt "$file")")
	echo "$name:$title:$content"
done
