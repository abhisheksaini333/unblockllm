import { NextRequest } from "next/server";

/** Optional: returns SVG favicon for state (normal | active | blocked). Use from client: /api/favicon?state=active */
export function GET(request: NextRequest) {
  const state = request.nextUrl.searchParams.get("state") || "normal";
  const svg =
    state === "active"
      ? '<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><rect x="6" y="6" width="20" height="20" rx="2" stroke="#16A34A" stroke-width="2" fill="none"/><path d="M12 16h6l-3-6" stroke="#16A34A" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>'
      : state === "blocked"
        ? '<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><rect x="6" y="6" width="20" height="20" rx="2" stroke="#DC2626" stroke-width="2" fill="none"/><path d="M10 10l12 12M22 10L10 22" stroke="#DC2626" stroke-width="2" stroke-linecap="round"/></svg>'
        : '<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><rect x="6" y="6" width="20" height="20" rx="2" stroke="#16A34A" stroke-width="2" fill="none"/><circle cx="16" cy="16" r="4" fill="#16A34A"/></svg>';

  return new Response(svg, {
    headers: {
      "Content-Type": "image/svg+xml",
      "Cache-Control": "public, max-age=0",
    },
  });
}
