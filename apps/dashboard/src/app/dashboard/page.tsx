"use client";

import { useSession } from "next-auth/react";
import { useRouter } from "next/navigation";
import { useEffect, useState } from "react";
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  CartesianGrid,
} from "recharts";
import Link from "next/link";

interface Stats {
  requests_by_day: { day: string; count: number }[];
  total_requests: number;
  total_entities_masked: number;
  by_entity_type: Record<string, number>;
}

export default function DashboardPage() {
  const { data: session, status } = useSession();
  const router = useRouter();
  const [stats, setStats] = useState<Stats | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (status === "unauthenticated") {
      router.push("/login?callbackUrl=/dashboard");
      return;
    }
    if (status !== "authenticated") return;
    fetch("/api/v1/stats?days=7")
      .then((r) => (r.ok ? r.json() : null))
      .then((d) => { setStats(d); setLoading(false); })
      .catch(() => setLoading(false));
  }, [status, router]);

  if (status === "loading" || !session) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <p className="text-gray-500">Loading...</p>
      </div>
    );
  }

  const chartData = (stats?.requests_by_day ?? []).map((r) => ({
    date: new Date(r.day).toLocaleDateString("en-US", { month: "short", day: "numeric" }),
    requests: r.count,
  }));

  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white border-b px-4 py-3 flex justify-between items-center">
        <h1 className="text-lg font-semibold">unblockllm Dashboard</h1>
        <div className="flex items-center gap-4">
          <span className="text-sm text-gray-600">{session.user?.email}</span>
          <Link href="/api/auth/signout" className="text-sm text-blue-600 hover:underline">Sign out</Link>
        </div>
      </header>
      <main className="max-w-4xl mx-auto p-6 space-y-6">
        <p className="text-sm text-gray-500">Aggregate usage only. No PII is stored or displayed.</p>
        {loading && <p>Loading stats...</p>}
        {!loading && stats && (
          <>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="bg-white p-4 rounded-lg shadow">
                <h2 className="text-sm font-medium text-gray-500">Requests (7 days)</h2>
                <p className="text-2xl font-semibold">{stats.total_requests}</p>
              </div>
              <div className="bg-white p-4 rounded-lg shadow">
                <h2 className="text-sm font-medium text-gray-500">PII entities masked</h2>
                <p className="text-2xl font-semibold">{stats.total_entities_masked}</p>
              </div>
            </div>
            {chartData.length > 0 && (
              <div className="bg-white p-4 rounded-lg shadow">
                <h2 className="text-sm font-medium text-gray-500 mb-4">Requests per day</h2>
                <ResponsiveContainer width="100%" height={280}>
                  <BarChart data={chartData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="date" />
                    <YAxis />
                    <Tooltip />
                    <Bar dataKey="requests" fill="#2563eb" name="Requests" />
                  </BarChart>
                </ResponsiveContainer>
              </div>
            )}
            {Object.keys(stats.by_entity_type ?? {}).length > 0 && (
              <div className="bg-white p-4 rounded-lg shadow">
                <h2 className="text-sm font-medium text-gray-500 mb-2">Entity types detected (count of requests containing type)</h2>
                <ul className="flex flex-wrap gap-2">
                  {Object.entries(stats.by_entity_type).map(([type, count]) => (
                    <li key={type} className="px-3 py-1 bg-gray-100 rounded text-sm">
                      {type}: {count}
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </>
        )}
        <div className="pt-4 border-t">
          <Link href="/dashboard/policy" className="text-blue-600 hover:underline">Edit policy (mask/block)</Link>
          {" · "}
          <Link href="/dashboard/billing" className="text-blue-600 hover:underline">Billing</Link>
        </div>
      </main>
    </div>
  );
}
