export interface Env {
  LEMONSQUEEZY_SIGNING_SECRET: string; // wrangler secret put LEMONSQUEEZY_SIGNING_SECRET
  RESEND_API_KEY: string;              // wrangler secret put RESEND_API_KEY
  LICENCE_MAGIC: string;               // wrangler secret put LICENCE_MAGIC  (e.g. "E4D1F00D")
  FROM_EMAIL: string;                  // e.g. "keys@envdiff.dev"
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    if (request.method !== "POST") {
      return new Response("Method not allowed", { status: 405 });
    }

    const body = await request.text();
    const signature = request.headers.get("X-Signature") ?? "";

    if (!(await verifySignature(body, signature, env.LEMONSQUEEZY_SIGNING_SECRET))) {
      return new Response("Invalid signature", { status: 401 });
    }

    const payload = JSON.parse(body);
    const event = payload.meta?.event_name;

    // Only handle paid orders
    if (event !== "order_created") {
      return new Response("OK", { status: 200 });
    }

    const attrs = payload.data?.attributes;
    if (attrs?.status !== "paid") {
      return new Response("OK", { status: 200 });
    }

    const email: string = attrs.user_email;
    const name: string = attrs.user_name ?? "there";
    const key = generateKey(env.LICENCE_MAGIC);

    await sendKeyEmail(email, name, key, env.RESEND_API_KEY, env.FROM_EMAIL);

    return new Response("OK", { status: 200 });
  },
};

// ── Key generation ────────────────────────────────────────────────────────────

function generateKey(magicHex: string): string {
  const magic = parseInt(magicHex, 16) >>> 0;
  const buf = new Uint32Array(2);
  crypto.getRandomValues(buf);
  const p1 = buf[0];
  const p2 = buf[1];
  const check = ((p1 ^ p2 ^ magic) & 0xffff) >>> 0;
  return `ENVD-${hex8(p1)}-${hex8(p2)}-${hex4(check)}`;
}

function hex8(n: number): string {
  return (n >>> 0).toString(16).toUpperCase().padStart(8, "0");
}

function hex4(n: number): string {
  return (n & 0xffff).toString(16).toUpperCase().padStart(4, "0");
}

// ── Webhook signature verification ────────────────────────────────────────────

async function verifySignature(body: string, signature: string, secret: string): Promise<boolean> {
  const enc = new TextEncoder();
  const key = await crypto.subtle.importKey(
    "raw",
    enc.encode(secret),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"]
  );
  const sig = await crypto.subtle.sign("HMAC", key, enc.encode(body));
  const expected = Array.from(new Uint8Array(sig))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
  return expected === signature;
}

// ── Email delivery via Resend ─────────────────────────────────────────────────

async function sendKeyEmail(
  email: string,
  name: string,
  key: string,
  resendApiKey: string,
  fromEmail: string
): Promise<void> {
  const res = await fetch("https://api.resend.com/emails", {
    method: "POST",
    headers: {
      Authorization: `Bearer ${resendApiKey}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      from: `envdiff <${fromEmail}>`,
      to: email,
      subject: "Your envdiff Pro licence key",
      html: `
        <div style="font-family:monospace;max-width:560px;margin:40px auto">
          <h2 style="color:#22c55e">envdiff Pro — activated</h2>
          <p>Hi ${name},</p>
          <p>Your licence key:</p>
          <pre style="background:#0f172a;color:#22c55e;padding:16px;border-radius:6px;font-size:18px;letter-spacing:2px">${key}</pre>
          <p>Run this to activate:</p>
          <pre style="background:#f1f5f9;padding:12px;border-radius:6px">envdiff activate ${key}</pre>
          <p>You now have access to <code>envdiff audit</code> and <code>envdiff guard</code>.</p>
          <p style="color:#64748b;font-size:13px">Questions? Just reply to this email.</p>
        </div>
      `,
    }),
  });

  if (!res.ok) {
    const err = await res.text();
    throw new Error(`Resend error: ${err}`);
  }
}
