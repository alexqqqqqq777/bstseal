import React from 'react';

const tiers = [
  {
    id: 'solo',
    name: 'Solo Developer',
    price: '$99 / year',
    features: ['4 CPU cores', 'Email support (48h)', 'Binary redistribution'],
  },
  {
    id: 'startup',
    name: 'Startup',
    price: '$999 / year',
    features: ['32 cores / 5k devices', 'Priority support (24h)', 'Private Slack'],
    highlight: true,
  },
];

export default function Pricing() {
  async function checkout(tier: string) {
    const res = await fetch('/api/checkout', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ tier }),
    });
    const { url } = await res.json();
    if (url) window.location.href = url;
  }

  return (
    <main className="px-6 py-20">
      <h1 className="text-4xl font-extrabold text-center mb-12">Pricing</h1>
      <div className="grid md:grid-cols-2 gap-8 max-w-4xl mx-auto">
        {tiers.map((t) => (
          <div
            key={t.id}
            className={`border rounded-lg p-6 flex flex-col ${t.highlight ? 'border-indigo-600 shadow-lg' : ''}`}
          >
            <h2 className="text-2xl font-semibold mb-2">{t.name}</h2>
            <p className="text-xl mb-4">{t.price}</p>
            <ul className="flex-1 mb-6 list-disc list-inside space-y-1 text-sm text-gray-700">
              {t.features.map((f) => (
                <li key={f}>{f}</li>
              ))}
            </ul>
            <button
              onClick={() => checkout(t.id)}
              className="w-full py-3 bg-indigo-600 text-white rounded font-medium"
            >
              Buy {t.name}
            </button>
          </div>
        ))}
      </div>
    </main>
  );
}
