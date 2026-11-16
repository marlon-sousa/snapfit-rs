import { chromium } from 'playwright';
import { diagramsDrawn, findPages, serve } from './serve.mjs';

const WIDTHS = [360, 390, 768, 1280];
const TOLERANCE = 1;

function measure() {
	const root = document.documentElement;
	const viewport = root.clientWidth;
	const main = document.querySelector('main')?.getBoundingClientRect();
	const wide = [
		...document.querySelectorAll('pre, .table-scroll, .diagram__scroll, .diagram:not(.diagram--wide) svg, .listing'),
	]
		.map((element) => ({ element, box: element.getBoundingClientRect() }))
		.filter(({ box }) => box.right > viewport + 1 || box.left < -1)
		.map(
			({ element }) =>
				element.getAttribute('aria-labelledby') ?? element.className ?? element.tagName.toLowerCase(),
		);
	return { viewport, scrollWidth: root.scrollWidth, mainLeft: main?.left ?? 0, mainRight: main?.right ?? 0, wide };
}

const pages = await findPages();
const { origin, close } = await serve();
const browser = await chromium.launch();
const problems = [];

await Promise.all(
	WIDTHS.map(async (width) => {
		const context = await browser.newContext({ viewport: { width, height: 800 } });
		const page = await context.newPage();
		for (const path of pages) {
			await page.goto(origin + path, { waitUntil: 'load' });
			await diagramsDrawn(page, 'light');
			const m = await page.evaluate(measure);
			if (m.scrollWidth > m.viewport + TOLERANCE) {
				problems.push(`${path} at ${width}px scrolls sideways (${m.scrollWidth}px > ${m.viewport}px).`);
			}
			if (m.mainLeft < -TOLERANCE || m.mainRight > m.viewport + TOLERANCE) {
				problems.push(`${path} at ${width}px has its main column outside the viewport.`);
			}
			if (m.wide.length > 0) {
				problems.push(`${path} at ${width}px has wider-than-viewport elements: ${m.wide.join(', ')}.`);
			}
		}
		await context.close();
	}),
);

await browser.close();
await close();

if (problems.length > 0) {
	console.error(`\n${problems.length} layout problem(s):\n`);
	for (const problem of problems) console.error(`  ${problem}`);
	process.exit(1);
}
console.log(`\nNo layout problems on ${pages.length} pages at ${WIDTHS.join(', ')} pixels wide.`);
