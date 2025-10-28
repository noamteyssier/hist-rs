
input_file := "./some.fq"
output_file := "./output.txt"

gen:
    cargo install nucgen; \
    nucgen -n 1000000 -l 20 {{input_file}}

bench:
    hyperfine \
        --warmup=10 \
        --export-markdown=benchmarks.md \
        --shell=bash \
        -n "hist" "hist <{{input_file}} >{{output_file}}" \
        -n "huniq" "huniq -cs <{{input_file}} >{{output_file}}" \
        -n "cuniq" "cuniq -cs <{{input_file}} >{{output_file}}" \
        -n "sortuniq" "sortuniq -c <{{input_file}} >{{output_file}}" \
        -n "naive-rust" "coreutils sort --parallel 1 <{{input_file}} | coreutils uniq -c | coreutils sort --parallel 1 -n >{{output_file}}" \
        -n "awk" "awk '{ x[$0]++ } END { for(y in x) { print y, x[y] }}' {{input_file}} | sort -k2,2nr > {{output_file}}" \
        -n "naive-no-locale" "export LC_ALL=C; sort < {{input_file}} | uniq -c | sort -n >{{output_file}}" \
        -n "naive-no-locale-size-hints" "export LC_ALL=C; sort < {{input_file}} -S 1G | uniq -c | sort -S 1G -n >{{output_file}}" \
        -n "naive-size-hints" "export LC_ALL=; sort < {{input_file}} -S 1G | uniq -c | sort  -S 1G -n >{{output_file}}" \
        -n "naive" "export LC_ALL=; sort < {{input_file}} | uniq -c | sort -n >{{output_file}}"

bench-stream:
    hyperfine \
        --warmup=10 \
        --export-markdown=benchmarks-stream.md \
        --shell=bash \
        -n "hist" "hist -u <{{input_file}} >{{output_file}}" \
        -n "huniq" "huniq <{{input_file}} >{{output_file}}" \
        -n "uq" "uq <{{input_file}} >{{output_file}}" \
        -n "ripuniq" "ripuniq <{{input_file}} >{{output_file}}" \
        -n "unic" "~/go/bin/unic <{{input_file}} >{{output_file}}" \
        -n "awk" "awk '!seen[$0]++' < {{input_file}} > {{output_file}}";
