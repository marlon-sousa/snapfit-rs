// @ts-check

/**
 * Strips `<!-- github-only -->` blocks, unwraps `<!-- site-only ... -->` blocks,
 * and replaces `<!-- snapfit:releases -->` with `releasesMarkdown`.
 *
 * @param {string} source
 * @param {string} releasesMarkdown
 * @returns {string}
 */
export function stripReadmeMarkdown(source, releasesMarkdown) {
	const lines = source.split('\n');
	/** @type {string[]} */
	const out = [];
	/** @type {'' | 'site' | 'github'} */
	let mode = '';
	let openedAt = 0;
	let sawReleasesMarker = false;

	for (let i = 0; i < lines.length; i++) {
		const line = lines[i].trim();
		const lineNo = i + 1;

		if (line === '<!-- site-only') {
			if (mode !== '') throw new Error(`README.md:${lineNo}: a site-only block opens inside another block.`);
			mode = 'site';
			openedAt = lineNo;
			continue;
		}
		if (line === '<!-- github-only -->') {
			if (mode !== '') throw new Error(`README.md:${lineNo}: a github-only block opens inside another block.`);
			mode = 'github';
			openedAt = lineNo;
			continue;
		}
		if (line === '-->') {
			if (mode !== 'site') throw new Error(`README.md:${lineNo}: --> closes no site-only block.`);
			mode = '';
			continue;
		}
		if (line === '<!-- end github-only -->') {
			if (mode !== 'github') throw new Error(`README.md:${lineNo}: <!-- end github-only --> closes no github-only block.`);
			mode = '';
			continue;
		}
		if (mode === 'github') continue;

		if (line === '<!-- snapfit:releases -->') {
			if (sawReleasesMarker) {
				throw new Error(`README.md:${lineNo}: holds a second <!-- snapfit:releases --> marker.`);
			}
			sawReleasesMarker = true;
			out.push(releasesMarkdown);
			continue;
		}
		out.push(lines[i]);
	}

	if (mode !== '') {
		throw new Error(`README.md:${openedAt}: ${mode} block is never closed.`);
	}
	if (!sawReleasesMarker) {
		throw new Error('README.md: missing <!-- snapfit:releases --> marker.');
	}
	return out.join('\n');
}
