using System;
using System.Runtime.InteropServices;

namespace Bstseal
{
    /// <summary>
    /// Unity-friendly C# wrapper for the BST-SEAL native library (libbstseal).
    /// Place the compiled binary (libbstseal.so / libbstseal.dylib / bstseal.dll)
    /// into a Plugins folder accessible to Unity.
    /// </summary>
    public static class Codec
    {
#if UNITY_IOS && !UNITY_EDITOR
        private const string DLL = "__Internal";
#else
        private const string DLL = "bstseal"; // Unity adds platform extension automatically
#endif

        private enum ErrorCode : int
        {
            Ok = 0,
            NullPointer = 1,
            EncodeFail = 2,
            DecodeFail = 3,
            IntegrityFail = 4,
            AllocFail = 5,
        }

        [DllImport(DLL, CallingConvention = CallingConvention.Cdecl)]
        private static extern int bstseal_encode(byte[] input, UIntPtr len, out IntPtr outPtr, out UIntPtr outLen);

        [DllImport(DLL, CallingConvention = CallingConvention.Cdecl)]
        private static extern int bstseal_decode(byte[] input, UIntPtr len, out IntPtr outPtr, out UIntPtr outLen);

        [DllImport(DLL, CallingConvention = CallingConvention.Cdecl)]
        private static extern void bstseal_free(IntPtr ptr);

        public static byte[] Encode(byte[] data)
        {
            if (data == null) throw new ArgumentNullException(nameof(data));
            IntPtr p; UIntPtr l;
            var code = bstseal_encode(data, (UIntPtr)data.Length, out p, out l);
            if (code != (int)ErrorCode.Ok)
                throw new Exception($"BST-SEAL encode failed: {(ErrorCode)code}");
            return CopyAndFree(p, l);
        }

        public static byte[] Decode(byte[] data)
        {
            if (data == null) throw new ArgumentNullException(nameof(data));
            IntPtr p; UIntPtr l;
            var code = bstseal_decode(data, (UIntPtr)data.Length, out p, out l);
            if (code != (int)ErrorCode.Ok)
                throw new Exception($"BST-SEAL decode failed: {(ErrorCode)code}");
            return CopyAndFree(p, l);
        }

        private static byte[] CopyAndFree(IntPtr ptr, UIntPtr len)
        {
            try
            {
                int length = checked((int)len);
                byte[] managed = new byte[length];
                Marshal.Copy(ptr, managed, 0, length);
                return managed;
            }
            finally
            {
                bstseal_free(ptr);
            }
        }
    }
}
