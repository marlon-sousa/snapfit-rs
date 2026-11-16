import type { APIRoute, GetStaticPaths } from 'astro';
import { base, build, markdownHref, site } from '../../../../book.config.mjs';
import { contents, pageName } from '../../../lib/contents.ts';

export const getStaticPaths: GetStaticPaths = () => [{ params: { version: build.version } }];

export const GET: APIRoute = () => {
	const { parts } = contents();
	const lines = [
		`# The Snapfit Book (${build.version})`,
		'',
		`> How to build with snapfit (${build.version}), a zero-cost, compile-time verified, pure-Rust dependency injection framework. Every example is a complete program that compiles and runs, with its recorded output shown underneath.`,
		'',
		`- [Contents](${site}${markdownHref()}): every part and chapter, in reading order.`,
		...parts.flatMap((part) => [
			'',
			`## ${part.title}`,
			'',
			...part.pages.map((page) => `- [${pageName(page)}](${site}${markdownHref(page.slug)})`),
		]),
		'',
		'## Optional',
		'',
		`- [snapfit Front Page](${site}${base}/index.md): overview, Why Snapfit, and releases.`,
		'',
	];
	return new Response(lines.join('\n'), { headers: { 'Content-Type': 'text/plain; charset=utf-8' } });
};
