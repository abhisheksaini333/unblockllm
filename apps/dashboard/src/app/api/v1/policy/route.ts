/**
 * Remote Policy API. Proxy polls this when DASHBOARD_URL is set.
 * Auth: X-Proxy-API-Key header must match PROXY_API_KEY.
 * No PII in response; only mask/block entity type names.
 */

import { NextRequest, NextResponse } from "next/server";
import { getPool, ensureSchema } from "@/lib/db";

export const dynamic = "force-dynamic";

function authProxyKey(req: NextRequest): boolean {
  const key = req.headers.get("x-proxy-api-key") ?? "";
  const expected = process.env.PROXY_API_KEY;
  if (!expected) return false;
  return key === expected;
}

export async function GET(req: NextRequest) {
  if (!authProxyKey(req)) {
    return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
  }
  try {
    await ensureSchema();
    const pool = getPool();
    const r = await pool.query("SELECT mask, block FROM policy_config WHERE id = 1");
    if (r.rows.length === 0) {
      return NextResponse.json({ mask: [], block: [] });
    }
    const row = r.rows[0];
    return NextResponse.json({
      mask: Array.isArray(row.mask) ? row.mask : [],
      block: Array.isArray(row.block) ? row.block : [],
    });
  } catch (e) {
    return NextResponse.json({ error: "Policy fetch failed" }, { status: 500 });
  }
}
