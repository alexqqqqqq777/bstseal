import React from 'react';
import Link from 'next/link';

export default function Home() {
  return (
    <main className="flex flex-col items-center px-6 py-20">
      <h1 className="text-5xl font-extrabold text-center mb-4">
        BST-SEAL™ <span className="text-indigo-600">blazing-fast</span> compression
      </h1>
      <p className="text-xl text-center max-w-2xl mb-8">
        Up to <strong>1.4× faster</strong> decode than Zstd while keeping comparable ratios.
        Patent pending technology ready for mission-critical workloads.
      </p>
      <div className="flex gap-4">
        <Link href="/pricing" className="px-6 py-3 rounded-lg bg-indigo-600 text-white font-medium shadow">
          Buy now
        </Link>
        <a href="https://github.com/your_org/bstseal" className="px-6 py-3 rounded-lg border font-medium">
          GitHub
        </a>
      </div>
      <section className="mt-20 max-w-4xl text-center">
        <h2 className="text-3xl font-bold mb-2">Benchmarks</h2>
        <p className="text-gray-600 mb-4">Intel i7-12700K · 4 MB file · mean of 5 runs</p>
        <div className="w-full bg-gray-200 rounded h-6 overflow-hidden">
          <div className="bg-indigo-600 h-full" style={{ width: '70%' }} />
        </div>
        <div className="flex justify-between text-sm mt-1">
          <span>Zstd (decode)</span>
          <span>BST-SEAL (decode)</span>
        </div>
      </section>
    </main>
  );
}
