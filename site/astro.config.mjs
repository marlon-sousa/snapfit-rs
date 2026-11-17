// @ts-check
import { join, normalize, sep } from 'node:path';
import { defineConfig } from 'astro/config';
import { base, bookDirectories, chapters, root, site } from './book.config.mjs';

function watchBookAndReadme() {
	const directories = bookDirectories.map((dir) => join(root, 'book', dir));
	const chapterDirectory = join(root, chapters);
	const contentsFile = join(chapterDirectory, 'README.md');
	const readmeFile = join(root, 'README.md');

	/** @param {string} file */
	const inWatchedDirs = (file) =>
		file === readmeFile || directories.some((dir) => file.startsWith(dir + sep));
	/** @param {string} file */
	const shapesBook = (file) => file.startsWith(chapterDirectory + sep);

	return {
		name: 'snapfit-site:watch',
		apply: /** @type {const} */ ('serve'),
		/** @param {import('vite').ViteDevServer} server */
		configureServer(server) {
			server.watcher.add([...directories, readmeFile]);
			server.watcher.on('change', (reported) => {
				const file = normalize(reported);
				if (file === contentsFile) server.restart();
				else if (inWatchedDirs(file)) server.ws.send({ type: 'full-reload', path: '*' });
			});
			/** @param {string} reported */
			const addedOrRemoved = (reported) => {
				const file = normalize(reported);
				if (shapesBook(file)) server.restart();
				else if (inWatchedDirs(file)) server.ws.send({ type: 'full-reload', path: '*' });
			};
			server.watcher.on('add', addedOrRemoved);
			server.watcher.on('unlink', addedOrRemoved);
		},
	};
}

export default defineConfig({
	site,
	base,
	trailingSlash: 'always',
	build: { format: 'directory' },
	vite: {
		plugins: [watchBookAndReadme()],
		build: { chunkSizeWarningLimit: 1000 },
	},
});
