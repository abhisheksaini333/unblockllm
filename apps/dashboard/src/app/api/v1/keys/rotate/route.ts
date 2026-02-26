/**
 * API Key Rotation (SEC-02). Requires session auth.
 * Generates a new PROXY_API_KEY replacement. Operator must update secret (e.g. K8s Secret)
 * and restart proxy; the raw key is returned once and not stored.
 */

import { NextResponse } from "next/server";
import { getServerSession } from "next-auth";
import { authOptions } from "@/lib/auth-options";
import { getPool, ensureSchema } from "@/lib/db";
import crypto from "crypto";

export const dynamic = "force-dynamic";

function generateSecureKey(): string {
  return "sk-" + crypto.randomBytes(32).toString("base64url");
}

export async function POST() {
  const session = await getServerSession(authOptions);
  if (!session?.user) {
    return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
  }
  try {
    await ensureSchema();
    const pool = getPool();
    const newKey = generateSecureKey();
    await pool.query(
      "INSERT INTO key_rotation_events (rotated_at) VALUES (NOW())"
    );
    return NextResponse.json({
      key: newKey,
      rotated_at: new Date().toISOString(),
      message:
        "Store this key in PROXY_API_KEY (e.g. K8s Secret / AWS Secrets Manager) and restart the proxy. Old key is invalid after you deploy.",
    });
  } catch (e) {
    return NextResponse.json(
      { error: "Key rotation failed" },
      { status: 500 }
    );
  }
}
