import Link from "next/link";
import { getServerSession } from "next-auth";
import { authOptions } from "@/lib/auth-options";
import { redirect } from "next/navigation";

export default async function Home() {
  const session = await getServerSession(authOptions);
  if (session) redirect("/dashboard");
  return (
    <main className="min-h-screen flex flex-col items-center justify-center p-8">
      <h1 className="text-2xl font-semibold mb-2">unblockllm Dashboard</h1>
      <p className="text-gray-500 mb-6">Zero-Trust Privacy Proxy control plane</p>
      <Link href="/login" className="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700">
        Sign in
      </Link>
    </main>
  );
}
