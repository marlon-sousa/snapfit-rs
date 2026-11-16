import type { APIRoute, GetStaticPaths } from 'astro';
import { build } from '../../../../book.config.mjs';
import { contents, type Page } from '../../../lib/contents.ts';
import { pageMarkdown } from '../../../lib/markdown.ts';

export const getStaticPaths: GetStaticPaths = () =>
	contents().pages.map((page) => ({
		params: { version: build.version, chapter: page.slug },
		props: { page },
	}));

export const GET: APIRoute = async ({ props }) => {
	const body = await pageMarkdown((props as { page: Page }).page);
	return new Response(body, { headers: { 'Content-Type': 'text/plain; charset=utf-8' } });
};
