
input_file := "./some.fq"
output_file := "./output.txt"

gen:
    cargo install nucgen; \
    nucgen -n 100000000 -l 20 {{input_file}}

bench:
    hyperfine \
        --warmup=10 \
        --export-markdown=benchmarks.md \
        -n "hist" "hist <{{input_file}} >{{output_file}}" \
        -n "huniq" "huniq -cs <{{input_file}} >{{output_file}}" \
        -n "cuniq" "cuniq -cs <{{input_file}} >{{output_file}}" \
        -n "sortuniq" "sortuniq -c <{{input_file}} >{{output_file}}" \
        -n "naive" "/bin/cat {{input_file}} | sort | uniq -c | sort -n >{{output_file}}"
