import { readFileSync } from "fs";
import { join } from "path";
import { notFound } from "next/navigation";
import ReactMarkdown from "react-markdown";
import Link from "next/link";

const SLUGS = ["quickstart", "security-model", "sla"] as const;

export function generateStaticParams() {
  return SLUGS.map((slug) => ({ slug }));
}

export default function DocPage({ params }: { params: { slug: string } }) {
  const slug = params.slug;
  if (!SLUGS.includes(slug as (typeof SLUGS)[number])) notFound();

  const contentPath = join(process.cwd(), "content", `${slug}.md`);
  let content: string;
  try {
    content = readFileSync(contentPath, "utf-8");
  } catch {
    notFound();
  }

  return (
    <div className="prose prose-invert max-w-none">
      <Link href="/" className="text-slate-400 hover:text-white text-sm mb-6 inline-block">
        ← Docs
      </Link>
      <ReactMarkdown>{content}</ReactMarkdown>
    </div>
  );
}
