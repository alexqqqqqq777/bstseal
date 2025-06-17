const path = require('path');
const os = require('os');
const ffi = require('ffi-napi');
const ref = require('ref-napi');

const voidPtr = ref.refType(ref.types.void);
const u8Ptr = ref.refType(ref.types.uint8);
const sizeT = ref.types.size_t;

function libPath() {
  const base = path.resolve(__dirname, '..', '..', 'target', 'release');
  switch (os.platform()) {
    case 'win32':
      return path.join(base, 'bstseal.dll');
    case 'darwin':
      return path.join(base, 'libbstseal.dylib');
    default:
      return path.join(base, 'libbstseal.so');
  }
}

const lib = ffi.Library(libPath(), {
  bstseal_encode: ['int', [u8Ptr, sizeT, ref.refType(u8Ptr), ref.refType(sizeT)]],
  bstseal_decode: ['int', [u8Ptr, sizeT, ref.refType(u8Ptr), ref.refType(sizeT)]],
  bstseal_free: ['void', [voidPtr]],
});

function callAndReturn(func, inputBuf) {
  const outPtrPtr = ref.alloc(u8Ptr);
  const outLenPtr = ref.alloc(sizeT);
  const code = lib[func](inputBuf, inputBuf.length, outPtrPtr, outLenPtr);
  if (code !== 0) {
    throw new Error(`${func} failed with code ${code}`);
  }
  const outPtr = outPtrPtr.deref();
  const outLen = outLenPtr.deref();
  const output = Buffer.from(ref.reinterpret(outPtr, outLen, 0));
  // copy to JS-managed buffer and free native memory
  const result = Buffer.from(output);
  lib.bstseal_free(outPtr);
  return result;
}

module.exports = {
  encode(buffer) {
    if (!Buffer.isBuffer(buffer)) throw new TypeError('buffer must be a Buffer');
    return callAndReturn('bstseal_encode', buffer);
  },
  decode(buffer) {
    if (!Buffer.isBuffer(buffer)) throw new TypeError('buffer must be a Buffer');
    return callAndReturn('bstseal_decode', buffer);
  },
};
