# File Comparison in SIMD

This is a terminal tool which similar to `cmp` in Linux, using SIMD from Rust to accelerate the comparison.

## How to run

``` bash
cargo run -- file1 file2
cargo run -- file1 file2 -s    # Silent mode - only return exit code
```

Or you can build your own binary by:

```bash
git clone https://github.com/Klaus-Guo/rcmp-simd.git
cd rcmp-simd
cargo build --release

# After these, there will be a binary at ./rcmp-simd/target/release/rcmp-simd, which is ready to be used.
```