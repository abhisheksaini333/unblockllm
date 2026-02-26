/**
 * Aggregate stats from Phase 3 audit_logs. No PII — only counts and types.
 * Auth: session or same X-Proxy-API-Key for server-to-server.
 */

import { NextRequest, NextResponse } from "next/server";
import { getServerSession } from "next-auth";
import { authOptions } from "@/lib/auth-options";
import { getPool, ensureSchema } from "@/lib/db";

export const dynamic = "force-dynamic";

function authProxyKey(req: NextRequest): boolean {
  const key = req.headers.get("x-proxy-api-key") ?? "";
  const expected = process.env.PROXY_API_KEY;
  return !!(expected && key === expected);
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
    // Ensure audit_logs exists (proxy creates it; dashboard may have run schema init before it was added here).
    await pool.query(`
      CREATE TABLE IF NOT EXISTS audit_logs (
        id BIGSERIAL PRIMARY KEY,
        request_id TEXT NOT NULL,
        user_id TEXT,
        entity_count INT NOT NULL,
        entity_types TEXT NOT NULL DEFAULT '[]',
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      )
    `);
    const days = Math.min(Math.max(1, parseInt(req.nextUrl.searchParams.get("days") ?? "7", 10)), 90);

    const requestsByDay = await pool.query(
      `SELECT date_trunc('day', created_at) AS day, COUNT(*) AS count
       FROM audit_logs
       WHERE created_at >= NOW() - ($1::text || ' days')::interval
       GROUP BY date_trunc('day', created_at)
       ORDER BY day ASC`,
      [days]
    );
    const totalRequests = await pool.query(
      `SELECT COUNT(*) AS total FROM audit_logs WHERE created_at >= NOW() - ($1::text || ' days')::interval`,
      [days]
    );
    const entityCounts = await pool.query(
      `SELECT SUM(entity_count) AS total_entities FROM audit_logs WHERE created_at >= NOW() - ($1::text || ' days')::interval`,
      [days]
    );

    const byType: Record<string, number> = {};
    const typesRow = await pool.query(
      `SELECT entity_types FROM audit_logs WHERE created_at >= NOW() - ($1::text || ' days')::interval`,
      [days]
    );
    for (const row of typesRow.rows) {
      const raw = row.entity_types;
      const arr = Array.isArray(raw) ? raw : (typeof raw === "string" ? (JSON.parse(raw || "[]") as string[]) : []);
      for (const t of arr) {
        byType[t] = (byType[t] ?? 0) + 1;
      }
    }

    return NextResponse.json({
      requests_by_day: requestsByDay.rows.map((r: { day: Date; count: string }) => ({ day: r.day, count: parseInt(r.count, 10) })),
      total_requests: parseInt(totalRequests.rows[0]?.total ?? "0", 10),
      total_entities_masked: parseInt(entityCounts.rows[0]?.total_entities ?? "0", 10) || 0,
      by_entity_type: byType,
    });
  } catch (e) {
    return NextResponse.json({ error: "Stats fetch failed" }, { status: 500 });
  }
}
