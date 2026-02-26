/**
 * Create Stripe Checkout Session for subscription. Plans: free (no checkout), pro, enterprise.
 */

import { NextRequest, NextResponse } from "next/server";
import { getServerSession } from "next-auth";
import Stripe from "stripe";
import { authOptions } from "@/lib/auth-options";
import { getPool, ensureSchema } from "@/lib/db";

export async function POST(req: NextRequest) {
  const session = await getServerSession(authOptions);
  if (!session?.user?.email) {
    return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
  }
  const secret = process.env.STRIPE_SECRET_KEY;
  const priceIdPro = process.env.STRIPE_PRICE_ID_PRO;
  const priceIdEnterprise = process.env.STRIPE_PRICE_ID_ENTERPRISE;
  if (!secret) {
    return NextResponse.json({ error: "Stripe not configured" }, { status: 503 });
  }
  const body = await req.json().catch(() => ({}));
  const plan = (body.plan as string) || "pro";
  const priceId = plan === "enterprise" ? priceIdEnterprise : priceIdPro;
  if (!priceId) {
    return NextResponse.json({ error: "Price not configured for plan" }, { status: 400 });
  }
  try {
    await ensureSchema();
    const pool = getPool();
    const userRow = await pool.query("SELECT id FROM users WHERE email = $1", [session.user.email]);
    if (userRow.rows.length === 0) return NextResponse.json({ error: "User not found" }, { status: 404 });
    const userId = userRow.rows[0].id;
    const baseUrl = process.env.NEXT_PUBLIC_APP_URL || process.env.NEXTAUTH_URL || "http://localhost:3000";
    const stripe = new Stripe(secret);
    const checkoutSession = await stripe.checkout.sessions.create({
      mode: "subscription",
      payment_method_types: ["card"],
      line_items: [{ price: priceId, quantity: 1 }],
      success_url: `${baseUrl}/dashboard?success=1`,
      cancel_url: `${baseUrl}/dashboard?canceled=1`,
      customer_email: session.user.email,
      client_reference_id: userId,
      metadata: { plan },
    });
    return NextResponse.json({ url: checkoutSession.url });
  } catch (e) {
    return NextResponse.json({ error: "Checkout failed" }, { status: 500 });
  }
}
