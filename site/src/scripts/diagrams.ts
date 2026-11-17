const WIDE = 2;

const figures = [...document.querySelectorAll<HTMLElement>('figure.diagram')];

if (figures.length > 0) {
	const dark = window.matchMedia('(prefers-color-scheme: dark)');
	const root = document.documentElement;
	let drawings = 0;

	let requested = 0;
	let drawn = 0;
	let drawing: Promise<void> | undefined;

	const request = () => {
		requested += 1;
		root.dataset.diagrams = 'drawing';
		drawing ??= drawUntilCurrent().finally(() => {
			drawing = undefined;
		});
	};

	const drawUntilCurrent = async () => {
		try {
			const { default: mermaid } = await import('mermaid');
			while (drawn < requested) {
				const current = requested;
				const scheme = dark.matches ? 'dark' : 'light';
				mermaid.initialize({
					startOnLoad: false,
					theme: 'base',
					themeVariables: palette(),
					securityLevel: 'strict',
					suppressErrorRendering: true,
				});
				for (const figure of figures) await drawOne(mermaid, figure);
				drawn = current;
				if (drawn === requested) root.dataset.diagrams = scheme;
			}
		} catch (error) {
			drawn = requested;
			root.dataset.diagrams = 'failed';
			console.error('The diagrams could not be drawn.', error);
		}
	};

	const palette = () => {
		const style = getComputedStyle(root);
		const token = (name: string) => style.getPropertyValue(`--${name}`).trim();
		return {
			darkMode: dark.matches,
			background: token('paper'),
			mainBkg: token('paper-raised'),
			primaryColor: token('paper-raised'),
			primaryTextColor: token('ink'),
			primaryBorderColor: token('rule-strong'),
			secondaryColor: token('paper-raised'),
			tertiaryColor: token('paper'),
			clusterBkg: token('paper'),
			clusterBorder: token('rule-strong'),
			lineColor: token('ink-muted'),
			textColor: token('ink'),
			titleColor: token('ink'),
			edgeLabelBackground: token('paper'),
			fontFamily: style.getPropertyValue('--font-ui').trim(),
		};
	};

	const drawOne = async (mermaid: typeof import('mermaid').default, figure: HTMLElement) => {
		const box = figure.querySelector<HTMLElement>('.diagram__scroll');
		const source = figure.querySelector<HTMLElement>('.diagram__source');
		if (!box || !source) return;
		drawings += 1;
		const { svg } = await mermaid.render(`diagram-drawing-${drawings}`, source.textContent ?? '');
		figure.querySelector('.diagram__drawing')?.remove();
		const holder = document.createElement('div');
		holder.className = 'diagram__drawing';
		holder.innerHTML = svg;
		box.append(holder);
		source.hidden = true;
		const picture = holder.querySelector('svg');
		nameByCaption(figure, picture);
		const [, , width, height] = (picture?.getAttribute('viewBox') ?? '').split(/\s+/).map(Number);
		if (width > WIDE * height) makeWide(figure, box);
	};

	const nameByCaption = (figure: HTMLElement, picture: SVGSVGElement | null) => {
		const caption = figure.querySelector('figcaption');
		if (!picture || !caption) return;
		picture.querySelector(':scope > title')?.remove();
		picture.setAttribute('aria-labelledby', caption.id);
	};

	const makeWide = (figure: HTMLElement, box: HTMLElement) => {
		figure.classList.add('diagram--wide');
		box.tabIndex = 0;
		box.setAttribute('role', 'group');
		const caption = figure.querySelector('figcaption');
		if (caption) box.setAttribute('aria-labelledby', caption.id);
	};

	request();
	dark.addEventListener('change', request);
}
