/**
 * Stripe webhook: update subscription status in DB. Verify signature with STRIPE_WEBHOOK_SECRET.
 */

import { NextRequest, NextResponse } from "next/server";
import Stripe from "stripe";
import { getPool, ensureSchema } from "@/lib/db";

export async function POST(req: NextRequest) {
  const secret = process.env.STRIPE_WEBHOOK_SECRET;
  if (!secret) {
    return NextResponse.json({ error: "Webhook secret not set" }, { status: 503 });
  }
  const body = await req.text();
  const sig = req.headers.get("stripe-signature");
  if (!sig) {
    return NextResponse.json({ error: "No signature" }, { status: 400 });
  }
  let event: Stripe.Event;
  try {
    const stripe = new Stripe(process.env.STRIPE_SECRET_KEY!);
    event = stripe.webhooks.constructEvent(body, sig, secret);
  } catch (err) {
    return NextResponse.json({ error: "Invalid signature" }, { status: 400 });
  }
  if (event.type !== "checkout.session.completed" && event.type !== "customer.subscription.updated" && event.type !== "customer.subscription.deleted") {
    return NextResponse.json({ received: true });
  }
  try {
    await ensureSchema();
    const pool = getPool();
    if (event.type === "checkout.session.completed") {
      const session = event.data.object as Stripe.Checkout.Session;
      const userId = session.client_reference_id;
      const plan = (session.metadata?.plan as string) || "pro";
      const customerId = session.customer as string;
      const subId = session.subscription as string;
      if (userId) {
        await pool.query(
          `INSERT INTO subscriptions (user_id, stripe_customer_id, stripe_subscription_id, plan, status)
           VALUES ($1, $2, $3, $4, 'active')
           ON CONFLICT (user_id) DO UPDATE SET stripe_customer_id = $2, stripe_subscription_id = $3, plan = $4, status = 'active', updated_at = NOW()`,
          [userId, customerId, subId, plan]
        );
      }
    } else if (event.type === "customer.subscription.updated" || event.type === "customer.subscription.deleted") {
      const sub = event.data.object as Stripe.Subscription;
      const status = sub.status === "active" ? "active" : "canceled";
      await pool.query(
        `UPDATE subscriptions SET status = $1, updated_at = NOW() WHERE stripe_subscription_id = $2`,
        [status, sub.id]
      );
    }
    return NextResponse.json({ received: true });
  } catch (e) {
    return NextResponse.json({ error: "Webhook handler failed" }, { status: 500 });
  }
}
