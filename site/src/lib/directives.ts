import { existsSync } from 'node:fs';
import { join, relative } from 'node:path';
import type { Code, PhrasingContent, Root, RootContent } from 'mdast';
import type { VFile } from 'vfile';
import { root, sourceHref } from '../../book.config.mjs';
import { readText } from './text.ts';

export interface DirectiveOptions {
	markdown?: boolean;
}

type Kind = 'example' | 'output' | 'compile-error';

const KEYS: Record<Kind, readonly string[]> = {
	example: ['name', 'region'],
	output: ['name'],
	'compile-error': ['name'],
};

const isKind = (lang: string | null | undefined): lang is Kind => lang != null && lang in KEYS;
const NAME = /^[a-z0-9_]+$/;

interface Directive {
	kind: Kind;
	name: string;
	region?: string;
}

function parse(node: Code, kind: Kind, chapter: string): Directive {
	const where = `${chapter}, line ${node.position?.start.line ?? '?'}`;
	const fields = new Map<string, string>();
	for (const line of node.value.split('\n')) {
		if (line.trim() === '') continue;
		const match = /^([a-z-]+):\s*(.+?)\s*$/.exec(line);
		if (!match) throw new Error(`${where}: "${line}" in a ${kind} directive is not a "key: value" line.`);
		const [, key, value] = match;
		if (!KEYS[kind].includes(key)) {
			throw new Error(`${where}: a ${kind} directive has no key "${key}". It takes ${KEYS[kind].join(', ')}.`);
		}
		if (!NAME.test(value)) {
			throw new Error(`${where}: "${value}" is not a name of lower-case letters, digits and underscores.`);
		}
		fields.set(key, value);
	}
	const name = fields.get('name');
	if (!name) throw new Error(`${where}: a ${kind} directive needs a name.`);
	return { kind, name, region: fields.get('region') };
}

const ANCHOR_LINE = /\bANCHOR(?:_END)?:\s*[\w-]+\s*$/;

function region(source: string, anchor: string | undefined, where: string): string {
	const lines = source.replace(/\n+$/, '').split('\n');
	let start = -1;
	let end = lines.length;
	if (anchor !== undefined) {
		start = lines.findIndex((line) => new RegExp(`\\bANCHOR:\\s*${anchor}\\s*$`).test(line));
		if (start === -1) throw new Error(`${where}: there is no "ANCHOR: ${anchor}".`);
		end = lines.findIndex((line, index) => index > start && new RegExp(`\\bANCHOR_END:\\s*${anchor}\\s*$`).test(line));
		if (end === -1) throw new Error(`${where}: "ANCHOR: ${anchor}" is never closed.`);
	}
	const kept = lines.slice(start + 1, end).filter((line) => !ANCHOR_LINE.test(line));
	if (kept.every((line) => line.trim() === '')) {
		throw new Error(anchor === undefined ? `${where}: the program is empty.` : `${where}: the region "${anchor}" is empty.`);
	}
	const indent = Math.min(...kept.filter((line) => line.trim() !== '').map((line) => /^\s*/.exec(line)?.[0].length ?? 0));
	return kept.map((line) => line.slice(indent)).join('\n');
}

function element(tagName: string, properties: Record<string, unknown>, children: unknown[]): RootContent {
	return { type: 'bookElement', data: { hName: tagName, hProperties: properties }, children } as unknown as RootContent;
}

const text = (value: string): PhrasingContent => ({ type: 'text', value });
const link = (url: string, value: string): PhrasingContent => ({ type: 'link', url, children: [text(value)] });

interface Listing {
	classes: string[];
	caption: PhrasingContent[];
	lang: string;
	code: string;
}

function codeLink(node: PhrasingContent): PhrasingContent {
	if (node.type !== 'link') return node;
	return {
		...node,
		children: node.children.map((child) => (child.type === 'text' ? { type: 'inlineCode', value: child.value } : child)),
	};
}

function plainCaption(caption: PhrasingContent[]): PhrasingContent[] {
	return caption.flatMap((node): PhrasingContent[] => {
		if ((node.type as string) !== 'bookElement') return [codeLink(node)];
		const { data, children } = node as unknown as { data: { hName: string }; children: PhrasingContent[] };
		return data.hName === 'strong' ? [{ type: 'strong', children }] : children;
	});
}

function plainListing(caption: PhrasingContent[], lang: string, code: string): RootContent[] {
	return [
		{ type: 'paragraph', children: plainCaption(caption) },
		{ type: 'code', lang, value: code },
	];
}

export default function remarkBookDirectives(options: DirectiveOptions = {}) {
	const { markdown = false } = options;
	const bookDir = join(root, 'book');
	const repoPath = (...parts: string[]) => ['book', ...parts].join('/');

	return (tree: Root, file: VFile) => {
		const chapter = file.path ? relative(root, file.path) : 'a chapter';
		let listings = 0;

		const read = (path: string, what: string) => {
			const full = join(bookDir, path);
			if (!existsSync(full)) {
				throw new Error(`${chapter}: ${what}, but ${repoPath(path)} does not exist.`);
			}
			return readText(full);
		};

		const figure = ({ classes, caption, lang, code }: Listing): RootContent[] => {
			if (markdown) return plainListing(caption, lang, code);
			listings += 1;
			const id = `listing-${listings}`;
			return [
				element('figure', { className: ['listing', ...classes] }, [
					element('figcaption', { id, className: ['listing__caption'] }, caption),
					{ type: 'code', lang, value: code } satisfies Code,
				]),
			];
		};

		const programName = (value: string): PhrasingContent => (markdown ? { type: 'inlineCode', value } : text(value));

		const programCaption = (lead: string, name: string, path: string, after: PhrasingContent[] = []) => [
			element('span', { className: ['listing__language'] }, [text('Rust')]) as PhrasingContent,
			text(lead),
			link(sourceHref(repoPath(path)), name),
			...after,
		];

		const marker = (label: string): PhrasingContent[] => [
			text('. '),
			element('strong', { className: ['listing__marker'] }, [text(label)]) as PhrasingContent,
		];

		const example = (directive: Directive): RootContent[] => {
			const path = `examples/${directive.name}.rs`;
			const code = region(
				read(path, `the example ${directive.name} is shown`),
				directive.region,
				`${chapter}: ${repoPath(path)}`,
			);
			return figure({
				classes: ['listing--example'],
				caption: programCaption(', from the program ', directive.name, path),
				lang: 'rust',
				code,
			});
		};

		const output = (directive: Directive): RootContent[] => {
			const path = `expected/${directive.name}.txt`;
			const outText = read(path, `an output directive names ${directive.name}`).replace(/\n+$/, '');
			return figure({
				classes: ['listing--output'],
				caption: [
					text('What '),
					programName(directive.name),
					text(' prints, recorded in '),
					link(sourceHref(repoPath(path)), `${directive.name}.txt`),
				],
				lang: 'text',
				code: outText,
			});
		};

		const compileError = (directive: Directive): RootContent[] => {
			const program = `compile-errors/${directive.name}.rs`;
			const code = region(
				read(program, `the compile error ${directive.name} is shown`),
				undefined,
				`${chapter}: ${repoPath(program)}`,
			);
			const message = read(`compile-errors/${directive.name}.txt`, `the compile error ${directive.name} is shown`).replace(
				/\n+$/,
				'',
			);
			return [
				...figure({
					classes: ['listing--example', 'listing--refused'],
					caption: programCaption(', the program ', directive.name, program, marker('Does not compile')),
					lang: 'rust',
					code,
				}),
				...figure({
					classes: ['listing--output', 'listing--compiler'],
					caption: [
						text('What the compiler says about '),
						programName(directive.name),
						text(', recorded in '),
						link(sourceHref(repoPath(`compile-errors/${directive.name}.txt`)), `${directive.name}.txt`),
					],
					lang: 'text',
					code: message,
				}),
			];
		};

		const render = (directive: Directive): RootContent[] => {
			switch (directive.kind) {
				case 'example':
					return example(directive);
				case 'output':
					return output(directive);
				case 'compile-error':
					return compileError(directive);
			}
		};

		const walk = (parent: { children: RootContent[] }) => {
			for (let index = 0; index < parent.children.length; index++) {
				const child = parent.children[index];
				if (child.type === 'code' && isKind(child.lang)) {
					const replacement = render(parse(child, child.lang, chapter));
					parent.children.splice(index, 1, ...replacement);
					index += replacement.length - 1;
				} else if ('children' in child) {
					walk(child);
				}
			}
		};

		walk(tree);
	};
}
