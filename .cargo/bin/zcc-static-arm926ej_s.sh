#!/usr/bin/env bash

new_array=()

for value in "$@"
do
    [[ "$value" != *self-contained/*crt* ]] && new_array+=("$value")
done

zig cc -g -fno-sanitize=all -target arm-linux-musleabi -march=arm926ej_s -mcpu=arm926ej_s "${new_array[@]}"
