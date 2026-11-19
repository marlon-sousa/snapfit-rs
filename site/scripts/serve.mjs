import { readFile, readdir } from 'node:fs/promises';
import { createServer } from 'node:http';
import { extname, join, relative } from 'node:path';
import { base } from '../book.config.mjs';

export const DIST = join(process.cwd(), 'dist');

const TYPES = {
	'.html': 'text/html; charset=utf-8',
	'.css': 'text/css; charset=utf-8',
	'.js': 'text/javascript; charset=utf-8',
	'.json': 'application/json',
	'.svg': 'image/svg+xml',
	'.png': 'image/png',
	'.md': 'text/plain; charset=utf-8',
	'.txt': 'text/plain; charset=utf-8',
};

export async function findPages(dir = DIST) {
	const pages = [];
	for (const entry of await readdir(dir, { withFileTypes: true })) {
		const full = join(dir, entry.name);
		if (entry.isDirectory()) {
			if (entry.name === '_astro') continue;
			pages.push(...(await findPages(full)));
		} else if (entry.name === 'index.html') {
			const path = relative(DIST, dir).split('\\').join('/');
			pages.push(path === '' ? `${base}/` : `${base}/${path}/`);
		}
	}
	return pages.sort();
}

export function fileFor(path, folder = DIST) {
	if (path !== base && !path.startsWith(`${base}/`)) return undefined;
	const inside = decodeURIComponent(path.slice(base.length)) || '/';
	return inside.endsWith('/') ? join(folder, inside, 'index.html') : join(folder, inside);
}

/**
 * The files a link to `path` may lead to: the one in this build, and, for a book edition other than
 * `edition`, the one in `published`, the site branch's pages, which keep every edition published before.
 *
 * @param {string} path
 * @param {string} edition The path of the edition this build makes, such as `/snapfit-rs/book/v0.1.1/`.
 * @param {string | undefined} published
 * @returns {string[]}
 */
export function linkTargets(path, edition, published) {
	const older = published && path.startsWith(`${base}/book/v`) && !path.startsWith(edition);
	return [fileFor(path), older ? fileFor(path, published) : undefined].filter((file) => file !== undefined);
}

export async function serve() {
	const server = createServer(async (request, response) => {
		const file = fileFor((request.url ?? '/').split('?')[0]);
		try {
			if (!file) throw new Error('outside the base');
			const body = await readFile(file);
			response.writeHead(200, { 'Content-Type': TYPES[extname(file)] ?? 'application/octet-stream' });
			response.end(body);
		} catch {
			response.writeHead(404, { 'Content-Type': 'text/html; charset=utf-8' });
			response.end(await readFile(join(DIST, '404.html')).catch(() => 'not found'));
		}
	});
	await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
	const address = server.address();
	const origin = `http://127.0.0.1:${typeof address === 'object' && address ? address.port : 0}`;
	return { origin, close: () => new Promise((resolve) => server.close(resolve)) };
}

export async function diagramsDrawn(page, scheme) {
	const state = await page.waitForFunction(
		(wanted) => {
			const state = document.documentElement.dataset.diagrams;
			return (state === undefined || state === wanted || state === 'failed') && (state ?? 'none');
		},
		scheme,
	);
	if ((await state.jsonValue()) === 'failed') {
		throw new Error(`${page.url()}: the diagrams could not be drawn; the browser console says why.`);
	}
}
