import { readFileSync } from 'node:fs';

export function readText(path: string): string {
	return readFileSync(path, 'utf8').replace(/\r\n/g, '\n');
}
