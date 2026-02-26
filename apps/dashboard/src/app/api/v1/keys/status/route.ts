/**
 * Key rotation status (SEC-02). For proxy or operators to check last rotation time.
 * Auth: session or X-Proxy-API-Key.
 */

import { NextRequest, NextResponse } from "next/server";
import { getServerSession } from "next-auth";
import { authOptions } from "@/lib/auth-options";
import { getPool, ensureSchema } from "@/lib/db";

export const dynamic = "force-dynamic";

function authProxyKey(req: NextRequest): boolean {
  const key = req.headers.get("x-proxy-api-key") ?? "";
  const expected = process.env.PROXY_API_KEY;
  if (!expected) return false;
  return key === expected;
}

export async function GET(req: NextRequest) {
  const session = await getServerSession(authOptions);
  const allowed = !!session?.user || authProxyKey(req);
  if (!allowed) {
    return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
  }
  try {
    await ensureSchema();
    const pool = getPool();
    const r = await pool.query(
      "SELECT rotated_at FROM key_rotation_events ORDER BY id DESC LIMIT 1"
    );
    if (r.rows.length === 0) {
      return NextResponse.json({ rotated_at: null });
    }
    return NextResponse.json({
      rotated_at: (r.rows[0].rotated_at as Date).toISOString(),
    });
  } catch (e) {
    return NextResponse.json(
      { error: "Status fetch failed" },
      { status: 500 }
    );
  }
}
