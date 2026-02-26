"use client";

import { useSession } from "next-auth/react";
import { useRouter } from "next/navigation";
import { useEffect, useState } from "react";
import Link from "next/link";

export default function PolicyPage() {
  const { data: session, status } = useSession();
  const router = useRouter();
  const [mask, setMask] = useState<string>("");
  const [block, setBlock] = useState<string>("");
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    if (status === "unauthenticated") {
      router.push("/login?callbackUrl=/dashboard/policy");
      return;
    }
    if (status !== "authenticated") return;
    fetch("/api/v1/policy/config")
      .then((r) => (r.ok ? r.json() : null))
      .then((d) => {
        if (d) {
          setMask(Array.isArray(d.mask) ? d.mask.join(", ") : "");
          setBlock(Array.isArray(d.block) ? d.block.join(", ") : "");
        }
      })
      .catch(() => {});
  }, [status, router]);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const maskArr = mask.split(",").map((s) => s.trim()).filter(Boolean);
    const blockArr = block.split(",").map((s) => s.trim()).filter(Boolean);
    const res = await fetch("/api/v1/policy/config", {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ mask: maskArr, block: blockArr }),
    });
    if (res.ok) setSaved(true);
  }

  if (status === "loading" || !session) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <p className="text-gray-500">Loading...</p>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white border-b px-4 py-3">
        <Link href="/dashboard" className="text-blue-600 hover:underline">← Dashboard</Link>
      </header>
      <main className="max-w-xl mx-auto p-6">
        <h1 className="text-xl font-semibold mb-4">Policy (mask / block)</h1>
        <p className="text-sm text-gray-500 mb-4">Entity types to mask (comma-separated). Empty = all. Block list rejects requests containing that type. No PII.</p>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700">Mask (e.g. EMAIL, PHONE, SSN)</label>
            <input
              type="text"
              value={mask}
              onChange={(e) => setMask(e.target.value)}
              className="mt-1 block w-full rounded border border-gray-300 px-3 py-2"
              placeholder="EMAIL, PHONE, PERSON, SSN"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700">Block (reject request if detected)</label>
            <input
              type="text"
              value={block}
              onChange={(e) => setBlock(e.target.value)}
              className="mt-1 block w-full rounded border border-gray-300 px-3 py-2"
              placeholder="SSN"
            />
          </div>
          <button type="submit" className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700">
            Save
          </button>
          {saved && <p className="text-sm text-green-600">Saved. Proxy will pick up on next poll.</p>}
        </form>
      </main>
    </div>
  );
}
