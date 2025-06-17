#ifndef BSTSEAL_C_H
#define BSTSEAL_C_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    BSTSEAL_OK = 0,
    BSTSEAL_NULL_POINTER = 1,
    BSTSEAL_ENCODE_FAIL = 2,
    BSTSEAL_DECODE_FAIL = 3,
    BSTSEAL_INTEGRITY_FAIL = 4,
    BSTSEAL_ALLOC_FAIL = 5,
} bstseal_error;

// Compresses `input[0..len)` into newly allocated buffer.
// On success returns BSTSEAL_OK and sets *out_ptr / *out_len.
// Caller must free *out_ptr via bstseal_free.
int bstseal_encode(const uint8_t* input, size_t len,
                   uint8_t** out_ptr, size_t* out_len);

// Decompresses buffer produced by bstseal_encode.
int bstseal_decode(const uint8_t* input, size_t len,
                   uint8_t** out_ptr, size_t* out_len);

// Frees memory returned from encode/decode.
void bstseal_free(void* ptr);

#ifdef __cplusplus
} // extern "C"
#endif

#endif /* BSTSEAL_C_H */
