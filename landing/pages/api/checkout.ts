import Stripe from 'stripe';
import type { NextApiRequest, NextApiResponse } from 'next';

const stripe = new Stripe(process.env.STRIPE_SECRET_KEY as string, {
  apiVersion: '2023-10-16',
});

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  if (req.method !== 'POST') {
    res.setHeader('Allow', 'POST');
    return res.status(405).end('Method Not Allowed');
  }

  const { tier } = req.body as { tier: 'solo' | 'startup' };
  const priceId = tier === 'startup' ? process.env.PRICE_STARTUP : process.env.PRICE_SOLO;
  if (!priceId) return res.status(500).json({ error: 'Price not configured' });

  try {
    const session = await stripe.checkout.sessions.create({
      mode: 'payment',
      line_items: [{ price: priceId, quantity: 1 }],
      success_url: `${req.headers.origin}/pricing?success=1`,
      cancel_url: `${req.headers.origin}/pricing?canceled=1`,
    });
    res.status(200).json({ url: session.url });
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Unknown error';
    res.status(500).json({ error: message });
  }
}
