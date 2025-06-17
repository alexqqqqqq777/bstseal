# BST-SEAL File & Archive Format (v1.0)

All multi-byte integers are little-endian. Offsets are counted from the beginning of the file.

## Single-block compressed stream (`.bsc`)

```
+---------------------+
| payload bytes       |  (compressed data produced by encoder)
+---------------------+
| 32-byte Blake3 hash |  (hash of payload, provides integrity)
+---------------------+
```

Decoder steps:
1. Read last 32 bytes, compute Blake3(payload) → compare.
2. If ok, feed `payload` into Huffman/RLE decoder.

## Archive container (`.bsa`)

```
+---------+-------------+-------------------+
| Field   | Size (bytes)| Description       |
+=========+=============+===================+
| MAGIC   | 8           | "BSTSEAL\0"       |
| count   | 4           | number of entries |
| entries | variable    | see below         |
| data    | variable    | concatenated encoded files |
+---------+-------------+-------------------+
```

### Entry structure (repeated `count` times, immediately after header)
```
+-------------+--------------+-----------------------------------------------+
| Field       | Size         | Description                                   |
+=============+==============+===============================================+
| path_len    | 2            | UTF-8 path length (bytes)                     |
| path        | path_len     | relative file path                            |
| offset      | 8            | start of compressed data within archive       |
| size        | 8            | length of compressed+footer segment           |
+-------------+--------------+-----------------------------------------------+
```

`offset` always points **after** the header; header length is computed before writing data.

The compressed segment is identical to the single-block stream (`payload || blake3`).

## Huffman sub-stream

* Code lengths array (256 bytes, values 0–15)
* (Option) future Package-Merge selector
* Big-endian bit-stream of codes (DEFLATE-style)

_Max code length_: 15 bits.

## Versioning

* Breaking header change bumps major version (encoded in `MAGIC` future extension).
* Minor/patch additions maintain backward compatibility.

## Reserved values

* `MAGIC` reserves last byte `0x00` for null-terminator, allowing `BSTSEAL\x01` for v2.
* Future flags/extensions will live after `count` field.

## Integrity & Security

* Blake3 provides collision-resistant verification.
* All numeric fields validated against file size to prevent OOB reads.
* Decoder uses bounded allocations; fast path avoids heap.
