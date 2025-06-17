# BST-SEAL™ *(patent pending)*

**BST-SEAL** (Block-Structured, Table-SEALed) is an experimental loss-less data-compression codec written in Rust.
It focuses on *blazing-fast* decode times on small/medium blocks while keeping compression ratios comparable to Zstandard.

Key features
------------
* SIMD-accelerated Huffman + RLE pipeline
* Blake3 footer for end-to-end integrity
* Multi-threaded encode / decode
* Archive mode (`.bsa`) with packing, listing, extraction & integrity check
* Cross-language bindings (C, Node.js, Unity/C#)
* Clean Rust API and production-ready CLI

Quick start (Rust / CLI)
-----------------------
```bash
# clone & build (release profile)
git clone https://github.com/your_org/bstseal.git
cd bstseal
cargo install --path crates/bstseal-cli # produces `bstseal` binary in $CARGO_HOME/bin

# compress / decompress single file
bstseal encode -i assets/small.bin -o small.bsc
bstseal decode -i small.bsc -o small.out

# work with archives
bstseal pack  -o data.bsa  assets/
bstseal list  data.bsa
bstseal cat   data.bsa assets/small.bin > /dev/null
bstseal unpack data.bsa -o extracted/

# verify integrity
bstseal fsck data.bsa
```

Using the Rust library
----------------------
```rust
use bstseal_core::{encode::encode_parallel, encode::decode_parallel};

let original = b"hello world";
let compressed = encode_parallel(original)?;
let decoded = decode_parallel(&compressed)?;
assert_eq!(original, decoded.as_slice());
```

C / C++ FFI
-----------
Header: `crates/bstseal-ffi/include/bstseal_c.h`
```c
#include "bstseal_c.h"

uint8_t* out; size_t out_len;
if (bstseal_encode(data, len, &out, &out_len) == BSTSEAL_OK) {
    /* use out/out_len */
    bstseal_free(out);
}
```
Shared library is produced by `cargo build -p bstseal-ffi --release`.

Node.js
-------
```bash
cd bindings/node
npm install                # compiles the native library automatically
```
```js
const bst = require('@your-org/bstseal');
const enc = bst.encode(Buffer.from('hello'));
const dec = bst.decode(enc);
```

Unity / C#
---------
Copy `libbstseal.(so|dylib|dll)` into `Assets/Plugins/` and add `bindings/unity/Bstseal.cs` to your scripts:
```csharp
byte[] compressed = Bstseal.Codec.Encode(data);
byte[] restored   = Bstseal.Codec.Decode(compressed);
```

Building & testing
------------------
```bash
cargo test --workspace --release   # Rust tests
npm --prefix bindings/node test     # JS smoke test
```
GitHub Actions workflow (`.github/workflows/ci.yml`) builds & lints on macOS/Ubuntu.

Benchmarks
----------
`bstseal bench -f sample.dat` prints encode/decode throughput vs block size.

License
-------
Open-source core: **MIT OR Apache-2.0** (dual-licensed).

Need to embed BST-SEAL in a closed-source or revenue-generating product?  A **Commercial License** is available – see [`LICENSE_COMMERCIAL.md`](LICENSE_COMMERCIAL.md) and the [Pricing table](PRICING.md).
