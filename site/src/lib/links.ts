import { posix } from 'node:path';
import type { Root } from 'mdast';
import { visit } from 'unist-util-visit';
import { chapters, pageHref, sourceHref } from '../../book.config.mjs';

const SCHEME = /^[a-z][a-z0-9+.-]*:/i;

export function resolveLink(url: string, from: string, pages: ReadonlySet<string>): string {
	if (url === '' || url.startsWith('#') || url.startsWith('//') || url.startsWith('/') || SCHEME.test(url)) return url;

	const hash = url.indexOf('#');
	const path = hash === -1 ? url : url.slice(0, hash);
	const fragment = hash === -1 ? '' : url.slice(hash);
	const target = posix.normalize(posix.join(posix.dirname(from), path));
	if (target.startsWith('..')) {
		throw new Error(`${from}: the link "${url}" leaves the repository.`);
	}

	if (target === `${chapters}/README.md`) return pageHref() + fragment;
	const chapter = target.startsWith(`${chapters}/`)
		? /^([a-z0-9-]+)\.md$/.exec(target.slice(chapters.length + 1))
		: null;
	if (chapter) {
		if (!pages.has(chapter[1])) {
			throw new Error(`${from}: the link "${url}" names ${target}, which is not a page of the book's contents.`);
		}
		return pageHref(chapter[1]) + fragment;
	}
	return sourceHref(target === '.' || target === './' ? '' : target) + fragment;
}

export function rewriteLinks(tree: Root, from: string, pages: ReadonlySet<string>): void {
	visit(tree, (node) => {
		if (node.type === 'link' || node.type === 'definition') {
			node.url = resolveLink(node.url, from, pages);
		}
	});
}
