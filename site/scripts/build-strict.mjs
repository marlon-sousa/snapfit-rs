import { spawn } from 'node:child_process';

const FATAL = ['Error rendering', 'Failed to parse', '[ERROR]'];

let output = '';
const child = spawn(process.execPath, ['./node_modules/astro/bin/astro.mjs', 'build', '--force'], {
	stdio: ['inherit', 'pipe', 'pipe'],
});
const watch = (chunk) => {
	output += chunk.toString();
	process.stdout.write(chunk);
};
child.stdout.on('data', watch);
child.stderr.on('data', watch);

child.on('close', (code) => {
	const offences = output
		.split(/\r?\n/)
		.filter((line) => FATAL.some((needle) => line.includes(needle)));
	if (offences.length > 0) {
		console.error('\nThe build printed errors. It did not succeed:\n');
		for (const line of offences) console.error(`  ${line.trim()}`);
		process.exit(1);
	}
	process.exit(code ?? 1);
});
