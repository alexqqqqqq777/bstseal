import { v4 as uuidv4 } from 'uuid';
import { createHmac } from 'crypto';

/**
 * Generates a signed license string that can be later verified without DB lookup.
 * Format: <uuid>.<tier>.<signature>
 * signature = base64url(HMAC_SHA256(uuid.tier, LICENSE_SECRET))
 */
export function generateLicense(tier: string): string {
  if (!process.env.LICENSE_SECRET) {
    throw new Error('LICENSE_SECRET env variable not set');
  }
  const id = uuidv4();
  const data = `${id}.${tier}`;
  const sig = createHmac('sha256', process.env.LICENSE_SECRET)
    .update(data)
    .digest('base64url');
  return `${data}.${sig}`;
}

export function verifyLicense(license: string): boolean {
  if (!process.env.LICENSE_SECRET) return false;
  const parts = license.split('.');
  if (parts.length < 3) return false;
  const sigProvided = parts.pop()!; // last part
  const data = parts.join('.');
  const sigExpected = createHmac('sha256', process.env.LICENSE_SECRET)
    .update(data)
    .digest('base64url');
  return sigProvided === sigExpected;
}
