import React from 'react';
import Script from 'next/script';

type Tier = {
  id: string;
  name: string;
  price: string;
  features: string[];
  hostedButtonId: string;
  highlight?: boolean;
};

const tiers: Tier[] = [
  {
    id: 'solo',
    name: 'Solo Developer',
    price: '$99 / year',
    features: ['4 CPU cores', 'Email support (48h)', 'Binary redistribution'],
    hostedButtonId: 'U83RDSRXY7EEN',
  },
  {
    id: 'startup',
    name: 'Startup',
    price: '$999 / year',
    features: ['32 cores / 5k devices', 'Priority support (24h)', 'Private Slack'],
    hostedButtonId: 'N6UCE6DK9D59C',
    highlight: true,
  },
];

export default function Pricing() {
  const renderButtons = () => {
    const paypal = (window as any).paypal;
    if (!paypal) return;
    tiers.forEach(t => {
      paypal.HostedButtons({ hostedButtonId: t.hostedButtonId }).render(`#paypal-container-${t.hostedButtonId}`);
    });
  };

  React.useEffect(() => {
    if (typeof window === 'undefined') return;
    // Try immediately (in case script already loaded)
    renderButtons();
  }, []);

  return (
    <>
      <Script
        src="https://www.paypal.com/sdk/js?client-id=BAA2tUyetolPtlwu57q1CCt_oUjyX_9lZqzW-oPFYvKKSBlVwVE3aF7Uie_dzhF0Nb-sCpQjB9y5aCVqnQ&components=hosted-buttons&disable-funding=venmo&currency=USD"
        strategy="afterInteractive" onLoad={() => renderButtons()}
      />
      <main className="px-6 py-20">
        <h1 className="text-4xl font-extrabold text-center mb-12">Pricing</h1>
        <div className="grid md:grid-cols-2 gap-8 max-w-4xl mx-auto">
          {tiers.map(t => (
            <div
              key={t.id}
              className={`border rounded-lg p-6 flex flex-col ${t.highlight ? 'border-indigo-600 shadow-lg' : ''}`}
            >
              <h2 className="text-2xl font-semibold mb-2">{t.name}</h2>
              <p className="text-xl mb-4">{t.price}</p>
              <ul className="flex-1 mb-6 list-disc list-inside space-y-1 text-sm text-gray-700">
                {t.features.map(f => (
                  <li key={f}>{f}</li>
                ))}
              </ul>
              <div id={`paypal-container-${t.hostedButtonId}`} />
            </div>
          ))}
        </div>
      </main>
    </>
  );
}
