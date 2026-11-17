import type { APIRoute } from 'astro';
import { releases } from '../../book.config.mjs';

export const GET: APIRoute = () =>
	new Response(JSON.stringify(releases, null, 2) + '\n', {
		headers: { 'Content-Type': 'application/json; charset=utf-8' },
	});
