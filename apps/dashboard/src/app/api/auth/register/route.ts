import { NextResponse } from "next/server";
import { createUser } from "@/lib/auth";
import { ensureSchema } from "@/lib/db";

export async function POST(req: Request) {
  try {
    const body = await req.json();
    const { email, password } = body as { email?: string; password?: string };
    if (!email || typeof email !== "string" || !password || typeof password !== "string") {
      return NextResponse.json({ error: "Email and password required" }, { status: 400 });
    }
    await ensureSchema();
    const user = await createUser(email, password);
    if (!user) return NextResponse.json({ error: "User already exists" }, { status: 409 });
    return NextResponse.json({ id: user.id, email: user.email, role: user.role });
  } catch (e) {
    return NextResponse.json({ error: "Registration failed" }, { status: 500 });
  }
}
