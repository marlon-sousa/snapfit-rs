import { join } from 'node:path';
import type { Root } from 'mdast';
import remarkGfm from 'remark-gfm';
import remarkParse from 'remark-parse';
import remarkStringify from 'remark-stringify';
import { unified } from 'unified';
import { visit } from 'unist-util-visit';
import { VFile } from 'vfile';
import { markdownHref, pageHref, root, site } from '../../book.config.mjs';
import { type Page, pageSlugs } from './contents.ts';
import remarkBookDirectives from './directives.ts';
import { rewriteLinks } from './links.ts';
import { readText } from './text.ts';

function absolute(url: string, slugs: ReadonlySet<string>): string {
	if (!url.startsWith('/')) return url;
	const hash = url.indexOf('#');
	const path = hash === -1 ? url : url.slice(0, hash);
	const fragment = hash === -1 ? '' : url.slice(hash);
	if (path === pageHref()) return `${site}${markdownHref()}${fragment}`;
	const page = [...slugs].find((slug) => path === pageHref(slug));
	return `${site}${page ? markdownHref(page) : path}${fragment}`;
}

function remarkMarkdownCopy(options: { from: string }) {
	return (tree: Root) => {
		const slugs = pageSlugs();
		rewriteLinks(tree, options.from, slugs);
		visit(tree, (node) => {
			if (node.type === 'link' || node.type === 'definition') node.url = absolute(node.url, slugs);
		});
	};
}

export async function markdownCopy(repositoryPath: string): Promise<string> {
	const file = new VFile({ value: readText(join(root, repositoryPath)), path: join(root, repositoryPath) });
	const processor = unified()
		.use(remarkParse)
		.use(remarkGfm)
		.use(remarkMarkdownCopy, { from: repositoryPath })
		.use(remarkBookDirectives, { markdown: true })
		.use(remarkStringify, { bullet: '-', fences: true, rule: '-' });
	return String(await processor.process(file));
}

export function pageMarkdown(page: Page): Promise<string> {
	return markdownCopy(page.file);
}
