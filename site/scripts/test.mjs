import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import test from 'node:test';
import { stripReadmeMarkdown } from '../src/lib/readme-strip.mjs';

test('stripReadmeMarkdown strips github-only, unwraps site-only, and expands releases marker', () => {
	const input = [
		'# snapfit',
		'<!-- site-only',
		'Site intro',
		'-->',
		'<!-- github-only -->',
		'![badge](x)',
		'<!-- end github-only -->',
		'<!-- snapfit:releases -->',
	].join('\n');

	const output = stripReadmeMarkdown(input, '| v0.1.0 |');
	assert.equal(output, ['# snapfit', 'Site intro', '| v0.1.0 |'].join('\n'));
});

test('stripReadmeMarkdown rejects unclosed, nested, or duplicate markers', () => {
	assert.throws(
		() => stripReadmeMarkdown('# snapfit\n<!-- github-only -->\nx\n<!-- snapfit:releases -->', 'rel'),
		/never closed/,
	);
	assert.throws(
		() => stripReadmeMarkdown('# snapfit\n<!-- site-only\n<!-- github-only -->\n-->', 'rel'),
		/opens inside another block/,
	);
	assert.throws(() => stripReadmeMarkdown('# snapfit\nno marker', 'rel'), /missing <!-- snapfit:releases -->/);
	assert.throws(
		() => stripReadmeMarkdown('# snapfit\n<!-- snapfit:releases -->\n<!-- snapfit:releases -->', 'rel'),
		/second <!-- snapfit:releases -->/,
	);
});

function findBash() {
	const candidates = process.platform === 'win32' ? ['C:\\Program Files\\Git\\bin\\bash.exe', 'bash'] : ['bash'];
	for (const candidate of candidates) {
		const res = spawnSync(candidate, ['--version'], { encoding: 'utf8' });
		if (res.status === 0) return candidate;
	}
	return null;
}

function makeFakeDist(dir, version, assetName) {
	mkdirSync(join(dir, 'book', version), { recursive: true });
	mkdirSync(join(dir, 'releases'), { recursive: true });
	mkdirSync(join(dir, '_astro'), { recursive: true });
	writeFileSync(join(dir, 'book', version, 'index.html'), `<h1>Book ${version}</h1>`);
	writeFileSync(join(dir, 'book', 'index.html'), `<h1>Books (latest ${version})</h1>`);
	writeFileSync(join(dir, 'releases', 'index.html'), `<h1>Releases (${version})</h1>`);
	writeFileSync(join(dir, 'index.html'), `<h1>snapfit (${version})</h1>`);
	writeFileSync(join(dir, 'index.md'), `# snapfit (${version})`);
	writeFileSync(join(dir, '404.html'), '<h1>404</h1>');
	writeFileSync(join(dir, 'favicon.svg'), '<svg></svg>');
	writeFileSync(join(dir, 'snapfit-site.json'), JSON.stringify([{ version }]));
	writeFileSync(join(dir, '_astro', assetName), `/* ${version} */`);
}

test('publish.sh creates orphan site branch and preserves older book editions and _astro assets', (t) => {
	const bash = findBash();
	if (!bash) {
		t.skip('bash is not available on this host');
		return;
	}

	const scratch = mkdtempSync(join(tmpdir(), 'snapfit-publish-test-'));
	try {
		const remote = join(scratch, 'remote.git');
		const repo = join(scratch, 'repo');
		const scriptSrc = resolve('scripts/publish.sh');

		execFileSync('git', ['init', '--bare', remote]);
		execFileSync('git', ['clone', remote, repo]);
		execFileSync('git', ['-C', repo, 'config', 'user.name', 'Test']);
		execFileSync('git', ['-C', repo, 'config', 'user.email', 'test@example.com']);
		writeFileSync(join(repo, 'README.md'), '# snapfit\n');
		execFileSync('git', ['-C', repo, 'add', 'README.md']);
		execFileSync('git', ['-C', repo, 'commit', '-m', 'init']);
		execFileSync('git', ['-C', repo, 'push', '-u', 'origin', 'HEAD']);

		const dist1 = join(scratch, 'dist-0.1.0');
		const wt1 = join(scratch, 'wt1');
		makeFakeDist(dist1, 'v0.1.0', 'base.v1.css');
		execFileSync(bash, [scriptSrc, 'open', wt1], { cwd: repo });
		execFileSync(bash, [scriptSrc, 'publish', wt1, dist1, 'v0.1.0'], { cwd: repo });

		const dist2 = join(scratch, 'dist-0.2.0');
		const wt2 = join(scratch, 'wt2');
		makeFakeDist(dist2, 'v0.2.0', 'base.v2.css');
		execFileSync(bash, [scriptSrc, 'open', wt2], { cwd: repo });
		execFileSync(bash, [scriptSrc, 'publish', wt2, dist2, 'v0.2.0'], { cwd: repo });

		const verifyClone = join(scratch, 'verify');
		execFileSync('git', ['clone', '--branch', 'site', remote, verifyClone]);

		assert.equal(existsSync(join(verifyClone, '.nojekyll')), true);
		assert.equal(readFileSync(join(verifyClone, 'book', 'v0.1.0', 'index.html'), 'utf8'), '<h1>Book v0.1.0</h1>');
		assert.equal(readFileSync(join(verifyClone, 'book', 'v0.2.0', 'index.html'), 'utf8'), '<h1>Book v0.2.0</h1>');
		assert.equal(readFileSync(join(verifyClone, '_astro', 'base.v1.css'), 'utf8'), '/* v0.1.0 */');
		assert.equal(readFileSync(join(verifyClone, '_astro', 'base.v2.css'), 'utf8'), '/* v0.2.0 */');
		assert.equal(readFileSync(join(verifyClone, 'index.html'), 'utf8'), '<h1>snapfit (v0.2.0)</h1>');

		assert.throws(
			() => execFileSync(bash, [scriptSrc, 'publish', wt2, dist2, '../bad'], { cwd: repo, stdio: 'pipe' }),
		);
	} finally {
		rmSync(scratch, { recursive: true, force: true });
	}
});
