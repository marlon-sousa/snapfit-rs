import type { APIRoute } from 'astro';
import { extractFrontPageMarkdown } from '../lib/readme.ts';

export const GET: APIRoute = () => {
	const body = extractFrontPageMarkdown(true);
	return new Response(body, { headers: { 'Content-Type': 'text/plain; charset=utf-8' } });
};
