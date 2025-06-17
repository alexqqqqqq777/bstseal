import Link from 'next/link';

export default function Navbar() {
  return (
    <header className="sticky top-0 z-50 backdrop-blur bg-white/70 border-b">
      <nav className="max-w-7xl mx-auto flex items-center justify-between px-6 py-3">
        <Link href="/" className="text-lg font-semibold tracking-tight text-gray-900">
          BST-SEAL<span className="text-indigo-600">™</span>
        </Link>
        <div className="flex items-center gap-6 text-sm font-medium">
          <Link href="/" className="text-gray-700 hover:text-indigo-600 transition-colors">Home</Link>
          <Link href="/pricing" className="text-gray-700 hover:text-indigo-600 transition-colors">Pricing</Link>
          <a href="https://github.com/alexqqqqqq777/bstseal" target="_blank" rel="noopener" className="text-gray-700 hover:text-indigo-600 transition-colors">GitHub</a>
        </div>
      </nav>
    </header>
  );
}
