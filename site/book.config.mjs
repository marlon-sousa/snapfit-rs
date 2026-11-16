// @ts-check
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';

/**
 * @typedef {object} ReleaseEntry
 * @property {string} version The release tag/version segment, such as `v0.1.0`.
 * @property {string} date ISO publication date (`YYYY-MM-DD`).
 * @property {string} summary One-sentence summary of the release.
 * @property {boolean} prerelease Whether this version is a pre-release.
 */

export const site = 'https://marlon-sousa.github.io';
export const base = '/wireup-rs';
export const repository = 'https://github.com/marlon-sousa/wireup-rs';
export const root = resolve(process.env.BOOK_ROOT ?? resolve(process.cwd(), '..'));

const SEGMENT = /^[a-zA-Z0-9][a-zA-Z0-9._+-]*$/;

function configured() {
	const version = process.env.BOOK_VERSION || 'v0.1.0';
	if (!SEGMENT.test(version)) {
		throw new Error(
			`BOOK_VERSION is "${version}". It must be one path segment of letters, digits, dots, hyphens, underscores or plus signs, such as "v0.1.0".`,
		);
	}
	return {
		version,
		sourceRef: process.env.BOOK_SOURCE_REF || 'main',
	};
}

export const build = configured();

/**
 * Reads a JSON file of release entries if it exists.
 * @param {string} filePath
 * @returns {ReleaseEntry[]}
 */
function readReleasesFile(filePath) {
	if (!existsSync(filePath)) return [];
	const parsed = JSON.parse(readFileSync(filePath, 'utf8'));
	if (!Array.isArray(parsed)) {
		throw new Error(`${filePath}: releases manifest must be a JSON array.`);
	}
	return parsed;
}

/**
 * Parses `vMAJOR.MINOR.PATCH[-PRE]` for newest-first sorting.
 * @param {string} version
 */
function parseSemver(version) {
	const clean = version.startsWith('v') ? version.slice(1) : version;
	const [core, pre = ''] = clean.split('-', 2);
	const parts = core.split('.').map((n) => Number.parseInt(n, 10));
	if (parts.length !== 3 || parts.some(Number.isNaN)) return null;
	return { major: parts[0], minor: parts[1], patch: parts[2], pre };
}

/**
 * Compares two releases newest-first by SemVer, then descending date/version.
 * @param {ReleaseEntry} a
 * @param {ReleaseEntry} b
 */
function compareReleasesNewestFirst(a, b) {
	const sa = parseSemver(a.version);
	const sb = parseSemver(b.version);
	if (sa && sb) {
		if (sa.major !== sb.major) return sb.major - sa.major;
		if (sa.minor !== sb.minor) return sb.minor - sa.minor;
		if (sa.patch !== sb.patch) return sb.patch - sa.patch;
		if (sa.pre === '' && sb.pre !== '') return -1;
		if (sa.pre !== '' && sb.pre === '') return 1;
		if (sa.pre !== sb.pre) return sb.pre.localeCompare(sa.pre);
	}
	if (a.date !== b.date) return b.date.localeCompare(a.date);
	return b.version.localeCompare(a.version);
}

function loadReleases() {
	/** @type {Map<string, ReleaseEntry>} */
	const byVersion = new Map();
	const manifestSources = [resolve(process.cwd(), 'releases.json')];
	if (process.env.WIREUP_RELEASES_FILE) {
		manifestSources.unshift(resolve(process.env.WIREUP_RELEASES_FILE));
	}
	for (const source of manifestSources) {
		for (const entry of readReleasesFile(source)) {
			byVersion.set(entry.version, entry);
		}
	}
	if (!byVersion.has(build.version)) {
		byVersion.set(build.version, {
			version: build.version,
			date: new Date().toISOString().slice(0, 10),
			summary: `wireup ${build.version} release and verified edition of The Wireup Book.`,
			prerelease: build.version.includes('-'),
		});
	}
	return [...byVersion.values()].sort(compareReleasesNewestFirst);
}

/** @type {readonly ReleaseEntry[]} */
export const releases = loadReleases();

export const chapters = 'book/chapters';
export const bookDirectories = ['chapters', 'examples', 'expected', 'compile-errors'];

/**
 * URL of a page in the current book edition (`/wireup-rs/book/<version>/...`).
 * @param {string} [page] Chapter slug without `.md`; omitted for the edition's contents.
 * @param {string} [ver] Optional version override.
 */
export function pageHref(page, ver = build.version) {
	return page ? `${base}/book/${ver}/${page}/` : `${base}/book/${ver}/`;
}

/**
 * URL of a page's Markdown copy (`/wireup-rs/book/<version>/<page>.md`).
 * @param {string} [page]
 * @param {string} [ver]
 */
export function markdownHref(page, ver = build.version) {
	return `${base}/book/${ver}/${page ?? 'index'}.md`;
}

/**
 * URL of a file in the GitHub repository.
 * @param {string} path Relative path in the repository, such as `book/examples/quick_start.rs`.
 */
export function sourceHref(path) {
	const kind = path === '' || path.endsWith('/') ? 'tree' : 'blob';
	return `${repository}/${kind}/${build.sourceRef}/${path}`;
}
