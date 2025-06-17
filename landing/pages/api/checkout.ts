
import type { NextApiRequest, NextApiResponse } from 'next';

export default function handler(req: NextApiRequest, res: NextApiResponse) {
  res.status(410).json({ error: 'Deprecated endpoint. Use PayPal buttons.' });
}
