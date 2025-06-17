import React from 'react';
import Link from 'next/link';

export default function Home() {
  return (
    <main className="flex flex-col items-center px-6 py-20">
      {/* HERO */}
      <h1 className="text-5xl md:text-6xl font-extrabold text-center mb-4 leading-tight">
        <span className="text-indigo-600">BST-SEAL</span> — the next generation
        <br className="hidden md:block" /> data compression engine
      </h1>
      <p className="text-xl md:text-2xl text-center max-w-3xl mb-10 text-gray-700">
        Decode up to <strong className="text-indigo-600">1.7× faster</strong> than Zstd<br />
        with the <strong>same or better</strong> compression ratios.
      </p>
      <div className="flex gap-4 mb-16 flex-wrap justify-center">
        <Link href="/pricing" className="px-8 py-3 rounded-lg bg-indigo-600 text-white font-medium shadow hover:bg-indigo-700 transition">Buy License</Link>
        <a href="https://github.com/your_org/bstseal" className="px-8 py-3 rounded-lg border font-medium hover:bg-gray-50 transition">GitHub</a>
        <a href="#benchmarks" className="px-8 py-3 rounded-lg border font-medium hover:bg-gray-50 transition">See Benchmarks</a>
      </div>

      {/* BENCHMARKS */}
      <section id="benchmarks" className="w-full max-w-5xl py-12">
        <h2 className="text-3xl font-bold text-center mb-6">Performance Benchmarks</h2>
        <p className="text-center text-gray-600 mb-8">Intel® i7-12700K · 32 MB dataset · average of 10 runs</p>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-6 text-center">
          <div className="p-4 border rounded-lg">
            <p className="text-sm text-gray-500 mb-1">Decode speed</p>
            <p className="text-2xl font-semibold">1.7×</p>
          </div>
          <div className="p-4 border rounded-lg">
            <p className="text-sm text-gray-500 mb-1">Encode speed</p>
            <p className="text-2xl font-semibold">1.2×</p>
          </div>
          <div className="p-4 border rounded-lg">
            <p className="text-sm text-gray-500 mb-1">Compression ratio</p>
            <p className="text-2xl font-semibold">≈ Zstd</p>
          </div>
          <div className="p-4 border rounded-lg">
            <p className="text-sm text-gray-500 mb-1">CPU usage</p>
            <p className="text-2xl font-semibold">-22 %</p>
          </div>
        </div>
      </section>

      {/* ADVANTAGES */}
      <section className="w-full max-w-5xl py-12">
        <h2 className="text-3xl font-bold text-center mb-6">Why BST-SEAL?</h2>
        <div className="grid md:grid-cols-3 gap-8">
          {[
            { title: 'Instant seek', desc: 'O(1) random access into compressed blocks — perfect for big-data analytics.' },
            { title: 'SIMD everywhere', desc: 'Hand-tuned AVX2/SVE + WebAssembly SIMD deliver top-tier speed on every platform.' },
            { title: 'Streaming-friendly', desc: 'Small memory footprint & back-pressure aware encoder suited for realtime pipelines.' },
            { title: 'Drop-in FFI', desc: 'C/Rust/Node bindings and a tiny CLI — integrate in minutes.' },
            { title: 'License-based', desc: 'Commercial-friendly license activation & monetisation built-in.' },
            { title: 'Open core', desc: 'Algorithm is open for inspection; pay only for production use & support.' },
          ].map((f) => (
            <div key={f.title} className="border rounded-lg p-6 text-center">
              <h3 className="text-xl font-semibold mb-2 text-indigo-600">{f.title}</h3>
              <p className="text-gray-700 text-sm leading-relaxed">{f.desc}</p>
            </div>
          ))}
        </div>
      </section>

      {/* USE CASES */}
      <section className="w-full max-w-5xl py-12">
        <h2 className="text-3xl font-bold text-center mb-6">Made for your workload</h2>
        <div className="grid md:grid-cols-2 gap-8">
          {[
            { h: 'Edge & IoT', p: 'Ultra-fast decode keeps latency low on ARM SBCs and micro-servers.' },
            { h: 'Streaming analytics', p: 'Push more telemetry per second without upgrading bandwidth.' },
            { h: 'Gen-AI pipelines', p: 'Feed models faster by decompressing tensors on-device.' },
            { h: 'WebAssembly', p: 'BST-SEAL WebAssembly build delivers sub-ms decompression in browsers.' },
          ].map((c) => (
            <div key={c.h} className="bg-gray-50 border rounded-lg p-6">
              <h4 className="text-lg font-semibold mb-1">{c.h}</h4>
              <p className="text-gray-700 text-sm leading-relaxed">{c.p}</p>
            </div>
          ))}
        </div>
      </section>

      {/* CTA */}
      <section className="w-full max-w-3xl py-16 text-center">
        <h2 className="text-3xl font-bold mb-4">Ready to turbo-charge your data?</h2>
        <p className="text-lg mb-6">Grab a license or try the open-core SDK today.</p>
        <div className="flex gap-4 justify-center">
          <Link href="/pricing" className="px-8 py-3 rounded-lg bg-indigo-600 text-white font-medium shadow hover:bg-indigo-700 transition">View Pricing</Link>
          <a href="https://github.com/your_org/bstseal" className="px-8 py-3 rounded-lg border font-medium hover:bg-gray-50 transition">Star on GitHub</a>
        </div>
      </section>
    </main>
  );

}
