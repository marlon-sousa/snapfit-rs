import { join } from 'node:path';
import rehypeShiki, { type RehypeShikiOptions } from '@shikijs/rehype';
import rehypeSlug from 'rehype-slug';
import rehypeStringify from 'rehype-stringify';
import remarkGfm from 'remark-gfm';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import type { Root } from 'mdast';
import { unified } from 'unified';
import { VFile } from 'vfile';
import { root } from '../../book.config.mjs';
import { type Page, pageSlugs } from './contents.ts';
import remarkBookDirectives from './directives.ts';
import { rewriteLinks } from './links.ts';
import { rehypeDiagrams, rehypeListingNames, rehypeNamedPreBlocks, rehypeTables } from './rehype.ts';
import { readText } from './text.ts';

function remarkBookLinks(options: { pages: () => ReadonlySet<string> }) {
	return (tree: Root, file: VFile) => {
		rewriteLinks(tree, String(file.data.repositoryPath), options.pages());
	};
}

let processor: ReturnType<typeof create> | undefined;

function create() {
	return unified()
		.use(remarkParse)
		.use(remarkGfm)
		.use(remarkBookLinks, { pages: pageSlugs })
		.use(remarkBookDirectives)
		.use(remarkRehype)
		.use(rehypeDiagrams)
		.use(rehypeShiki, {
			themes: { light: 'github-light-default', dark: 'github-dark-default' },
			defaultColor: 'light',
			langs: ['rust', 'toml', 'markdown', 'sh'] as RehypeShikiOptions['langs'],
		})
		.use(rehypeListingNames)
		.use(rehypeNamedPreBlocks)
		.use(rehypeTables)
		.use(rehypeSlug)
		.use(rehypeStringify);
}

export async function renderMarkdown(source: string, repositoryPath: string): Promise<string> {
	processor ??= create();
	const file = new VFile({ value: source, path: join(root, repositoryPath), data: { repositoryPath } });
	return String(await processor.process(file));
}

export interface RenderedPage {
	title: string;
	html: string;
}

export async function renderPage(page: Page): Promise<RenderedPage> {
	const source = readText(join(root, page.file));
	const heading = /^# (.+)$/m.exec(source);
	if (!heading) throw new Error(`${page.file}: the page has no level 1 heading.`);
	return { title: heading[1].replace(/`/g, '').trim(), html: await renderMarkdown(source, page.file) };
}
