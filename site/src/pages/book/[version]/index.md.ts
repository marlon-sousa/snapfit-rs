import type { APIRoute, GetStaticPaths } from 'astro';
import { build, chapters } from '../../../../book.config.mjs';
import { markdownCopy } from '../../../lib/markdown.ts';

export const getStaticPaths: GetStaticPaths = () => [{ params: { version: build.version } }];

export const GET: APIRoute = async () => {
	const body = await markdownCopy(`${chapters}/README.md`);
	return new Response(body, { headers: { 'Content-Type': 'text/plain; charset=utf-8' } });
};
