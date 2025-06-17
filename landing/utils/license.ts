import { v4 as uuidv4 } from 'uuid';
import { createHmac } from 'crypto';

/**
 * Generates a signed license string that can be later verified without DB lookup.
 * Format: <uuid>.<tier>.<expires_iso>.<signature>
 * expires_iso = ISO string (UTC) when license stops being valid (e.g. now+1y)
 * signature = base64url(HMAC_SHA256(uuid.tier.expires_iso, LICENSE_SECRET))
 */
export function generateLicense(tier: string, validityDays = 365): string {
  if (!process.env.LICENSE_SECRET) {
    throw new Error('LICENSE_SECRET env variable not set');
  }
  const id = uuidv4();
  const expires = new Date(Date.now() + validityDays * 24 * 60 * 60 * 1000).toISOString();
  const data = `${id}.${tier}.${expires}`;
  const sig = createHmac('sha256', process.env.LICENSE_SECRET)
    .update(data)
    .digest('base64url');
  return `${data}.${sig}`;
}

export function verifyLicense(license: string): boolean {
  if (!process.env.LICENSE_SECRET) return false;
  const parts = license.split('.');
  if (parts.length < 4) return false;
  const sigProvided = parts.pop()!; // last part
  const data = parts.join('.');
  const sigExpected = createHmac('sha256', process.env.LICENSE_SECRET)
    .update(data)
    .digest('base64url');
  return sigProvided === sigExpected;
}
