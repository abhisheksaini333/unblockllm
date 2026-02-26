/**
 * Dashboard policy config: GET/PUT for authenticated users. No PII.
 */

import { NextRequest, NextResponse } from "next/server";
import { getServerSession } from "next-auth";
import { authOptions } from "@/lib/auth-options";
import { getPool, ensureSchema } from "@/lib/db";

export const dynamic = "force-dynamic";

export async function GET() {
  const session = await getServerSession(authOptions);
  if (!session?.user) return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
  try {
    await ensureSchema();
    const pool = getPool();
    const r = await pool.query("SELECT mask, block FROM policy_config WHERE id = 1");
    if (r.rows.length === 0) return NextResponse.json({ mask: [], block: [] });
    const row = r.rows[0];
    return NextResponse.json({
      mask: Array.isArray(row.mask) ? row.mask : [],
      block: Array.isArray(row.block) ? row.block : [],
    });
  } catch (e) {
    return NextResponse.json({ error: "Fetch failed" }, { status: 500 });
  }
}

export async function PUT(req: NextRequest) {
  const session = await getServerSession(authOptions);
  if (!session?.user) return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
  const body = await req.json().catch(() => ({}));
  const mask = Array.isArray(body.mask) ? body.mask : [];
  const block = Array.isArray(body.block) ? body.block : [];
  try {
    await ensureSchema();
    const pool = getPool();
    await pool.query(
      `UPDATE policy_config SET mask = $1, block = $2, updated_at = NOW() WHERE id = 1`,
      [JSON.stringify(mask), JSON.stringify(block)]
    );
    return NextResponse.json({ ok: true });
  } catch (e) {
    return NextResponse.json({ error: "Update failed" }, { status: 500 });
  }
}
