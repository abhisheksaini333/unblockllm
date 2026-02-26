import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

/**
 * SEC-03: Enforce HTTPS in production.
 * - When behind a TLS-terminating LB: redirect if x-forwarded-proto is "http" (307).
 * - When accessed directly: redirect if request URL is http (307).
 * CORS: API routes are same-origin by default; no permissive ACAO.
 */
export function middleware(request: NextRequest) {
  if (process.env.NODE_ENV !== "production") {
    return NextResponse.next();
  }
  const proto = request.headers.get("x-forwarded-proto");
  const url = request.nextUrl.clone();
  const isHttp =
    proto === "http" || (url.protocol === "http:" && url.hostname !== "localhost");
  if (isHttp) {
    url.protocol = "https:";
    return NextResponse.redirect(url, 307);
  }
  return NextResponse.next();
}

export const config = {
  matcher: ["/((?!api|_next/static|_next/image|favicon.ico).*)"],
};
