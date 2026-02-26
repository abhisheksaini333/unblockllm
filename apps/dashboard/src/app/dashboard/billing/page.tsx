"use client";

import { useSession } from "next-auth/react";
import { useRouter } from "next/navigation";
import { useEffect, useState } from "react";
import Link from "next/link";

export default function BillingPage() {
  const { data: session, status } = useSession();
  const router = useRouter();
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (status === "unauthenticated") {
      router.push("/login?callbackUrl=/dashboard/billing");
      return;
    }
  }, [status, router]);

  async function handleCheckout(plan: string) {
    setLoading(true);
    try {
      const res = await fetch("/api/stripe/checkout", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ plan }),
      });
      const data = await res.json();
      if (data.url) window.location.href = data.url;
      else alert("Checkout not available");
    } finally {
      setLoading(false);
    }
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
        <h1 className="text-xl font-semibold mb-4">Billing</h1>
        <p className="text-sm text-gray-500 mb-6">Plans: Free (default), Pro, Enterprise.</p>
        <div className="space-y-4">
          <button
            onClick={() => handleCheckout("pro")}
            disabled={loading}
            className="w-full bg-blue-600 text-white py-2 rounded hover:bg-blue-700 disabled:opacity-50"
          >
            Subscribe to Pro
          </button>
          <button
            onClick={() => handleCheckout("enterprise")}
            disabled={loading}
            className="w-full bg-gray-700 text-white py-2 rounded hover:bg-gray-800 disabled:opacity-50"
          >
            Subscribe to Enterprise
          </button>
        </div>
      </main>
    </div>
  );
}
