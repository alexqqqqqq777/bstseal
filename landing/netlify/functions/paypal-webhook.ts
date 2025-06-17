import type { Handler } from "@netlify/functions";
import { generateLicense } from "../../utils/license";
import { Resend } from "resend";

// Minimal mapping between PayPal Hosted Button IDs and tier names
const TIER_BY_BUTTON: Record<string, string> = {
  U83RDSRXY7EEN: "solo",
  N6UCE6DK9D59C: "startup",
};

/**
 * Utility: extract hostedButtonId from webhook payload.
 * For Hosted Buttons PayPal adds it under resource.custom_id or purchase_units[0].custom_id.
 */
function extractButtonId(payload: any): string | undefined {
  return (
    payload?.resource?.custom_id ||
    payload?.resource?.purchase_units?.[0]?.custom_id ||
    payload?.resource?.supplementary_data?.related_ids?.button_id
  );
}

export const handler: Handler = async (event, context) => {
  if (event.httpMethod !== "POST") {
    return { statusCode: 405, body: "Method Not Allowed" };
  }

  if (!process.env.RESEND_API_KEY) {
    console.error("RESEND_API_KEY not set");
    return { statusCode: 500, body: "Missing email API key" };
  }

  let payload: any;
  try {
    payload = JSON.parse(event.body ?? "{}");
  } catch (e) {
    console.error("Invalid JSON", e);
    return { statusCode: 400, body: "Invalid JSON" };
  }

  const eventType = payload?.event_type;
  if (
    eventType !== "PAYMENT.CAPTURE.COMPLETED" &&
    eventType !== "CHECKOUT.ORDER.APPROVED"
  ) {
    // Not a payment completion event we care about
    return { statusCode: 200, body: "Ignored" };
  }

  // TODO: proper PayPal webhook signature verification
  // For MVP we skip verification – enable in production.

  const hostedButtonId = extractButtonId(payload);
  const tier = (hostedButtonId && TIER_BY_BUTTON[hostedButtonId]) || "unknown";

  const payerEmail =
    payload?.resource?.payer?.email_address ||
    payload?.resource?.payer?.payer_info?.email ||
    payload?.resource?.payer?.email;

  if (!payerEmail) {
    console.error("No payer email in webhook payload");
    return { statusCode: 200, body: "No email" };
  }

  // Generate license
  let license: string;
  try {
    license = generateLicense(tier);
  } catch (err) {
    console.error(err);
    return { statusCode: 500, body: "License generation failed" };
  }

  // Send email via Resend
  try {
    const resend = new Resend(process.env.RESEND_API_KEY);
    await resend.emails.send({
      from: "BST-SEAL <noreply@bstseal.dev>",
      to: payerEmail,
      subject: "Your BST-SEAL License Key",
      html: `<p>Hi!</p><p>Thanks for purchasing the <strong>${tier}</strong> plan.</p><p>Your license key:</p><pre style="font-size:16px;">${license}</pre><p>Keep it safe – you'll need it to activate the library.</p><p>Best regards,<br/>BST-SEAL team</p>`,
    });
    console.log(`License sent to ${payerEmail}`);
  } catch (err) {
    console.error("Failed to send email", err);
    return { statusCode: 500, body: "Email send failed" };
  }

  // Success
  return { statusCode: 200, body: "ok" };
};
