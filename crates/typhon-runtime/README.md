# typhon-runtime

Runtime support library for the Typhon programming language.

This crate provides the runtime system including memory management, garbage collection, and core runtime services for executing Typhon programs.

## Building the Runtime Library

### Prerequisites

- C compiler (clang or gcc)
- ar (archive tool for creating static libraries)

### Build Instructions

1. Create the `build_runtime.sh` script in this directory:

    ```bash
    #!/usr/bin/env bash
    #
    # Build script for Typhon runtime library
    # Compiles runtime.c into a static library for linking with Typhon programs

    set -euo pipefail  # Exit on error

    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    cd "$SCRIPT_DIR"

    echo "Building Typhon runtime library..."
    echo "Directory: $SCRIPT_DIR"

    # Compile runtime.c to object file
    echo "Step 1: Compiling runtime.c to object file..."
    clang -c runtime.c -o libtyphon_rt.o -O2 -Wall -Wextra

    # Verify object file was created
    if [ ! -f "libtyphon_rt.o" ]; then
        echo "ERROR: Failed to create libtyphon_rt.o"
        exit 1
    fi

    echo "Step 2: Creating static library..."
    ar rcs libtyphon_rt.a libtyphon_rt.o

    # Verify static library was created
    if [ ! -f "libtyphon_rt.a" ]; then
        echo "ERROR: Failed to create libtyphon_rt.a"
        exit 1
    fi

    echo "Step 3: Verifying symbols in library..."
    echo ""
    echo "Symbols defined in libtyphon_rt.a:"

    nm libtyphon_rt.a | grep " T " || echo "  (none found)"

    echo ""
    echo "All symbols:"
    nm libtyphon_rt.a
    echo ""

    echo "✓ Runtime library built successfully!"
    echo "  - Object file: libtyphon_rt.o"
    echo "  - Static library: libtyphon_rt.a"
    echo ""
    echo "To link with a Typhon program:"
    echo "  clang program.o crates/typhon-runtime/libtyphon_rt.a -o program"
    ```

2. Make the script executable and run it:

    ```shell
    # Make the build script executable (first time only)
    chmod +x crates/typhon-runtime/build_runtime.sh

    # Build the runtime library
    ./crates/typhon-runtime/build_runtime.sh
    ```

### Build Output

The build script creates:

- `crates/typhon-runtime/libtyphon_rt.o` - Object file
- `crates/typhon-runtime/libtyphon_rt.a` - Static library (use this for linking)

### Verifying the Build

```shell
# List symbols in the library
nm crates/typhon-runtime/libtyphon_rt.a | grep " T "

# Expected output:
# 0000000000000004 T _typhon_add
# 0000000000000000 T _typhon_int_new
# 000000000000000c T _typhon_print
```

## Linking Typhon Programs

### Basic Linking

To link a compiled Typhon object file with the runtime:

```bash
clang program.o crates/typhon-runtime/libtyphon_rt.a -o program
```
