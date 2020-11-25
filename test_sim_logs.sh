#!/bin/bash -e

# 

SCRIPT=./target/release/spike-dasm-rs
IN_DIR=inputs/sim-logs-no-dasm
GOLDEN_DIR=inputs/sim-logs-dasm
OUT_DIR=tmp-dasm

[[ ! -d "$IN_DIR" ]] && echo "Couldn't find input directory '$IN_DIR'"
[[ ! -d "$GOLDEN_DIR" ]] && echo "Couldn't find golden directory '$GOLDEN_DIR'"

[[ -d "$OUT_DIR" ]] && rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

cargo build --release
[[ ! -f "$SCRIPT" ]] && echo "Couldn't find test program '$SCRIPT'" && exit 1

for in_f in $IN_DIR/* ; do
    f_name="$(basename $in_f)"
    golden_f="$GOLDEN_DIR/$f_name"
    out_f="$OUT_DIR/$f_name"
    [[ ! -f "$golden_f" ]] && echo "Couldn't find golden file '$golden_f'" && exit 1

    echo "Testing $f_name... "
    $SCRIPT < "$in_f" > "$out_f"

    # Compare the output to the expected result. Skip the first 3 lines in the
    # file, as they contain a few random numbers which will differ.
    diff <(tail -n +4 "$out_f") <(tail -n +4 "$golden_f")

    rm "$out_f"
done

rm -rf "$OUT_DIR"

echo "Passed!"
