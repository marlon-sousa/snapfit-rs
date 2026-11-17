import { join } from 'node:path';
import { base, pageHref, releases, repository, root, site } from '../../book.config.mjs';
import { stripReadmeMarkdown } from './readme-strip.mjs';
import { readText } from './text.ts';

function buildReleasesMarkdown(absolute: boolean): string {
	const prefix = absolute ? site : '';
	const rows = releases.map((rel) => {
		const bookUrl = `${prefix}${pageHref(undefined, rel.version)}`;
		const notesUrl = `${repository}/releases/tag/${rel.version}`;
		return `| **${rel.version}** (${rel.date}) | [Read the Book (${rel.version})](${bookUrl}) | [Release notes on GitHub](${notesUrl}) | ${rel.summary} |`;
	});

	return [
		'| Version | Book Edition | Release Notes | Highlights |',
		'| :--- | :--- | :--- | :--- |',
		...rows,
		'',
		`Browse [all releases and editions](${prefix}${base}/releases/).`,
	].join('\n');
}

/**
 * Extracts the front page Markdown from the root README.md.
 */
export function extractFrontPageMarkdown(absolute = false): string {
	const source = readText(join(root, 'README.md'));
	return stripReadmeMarkdown(source, buildReleasesMarkdown(absolute));
}
