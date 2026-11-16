import type { Element, Root } from 'hast';
import { toString } from 'hast-util-to-string';
import type { VFile } from 'vfile';
import { visit } from 'unist-util-visit';

const hasClass = (node: Element, name: string) => {
	const classes = node.properties.className;
	return Array.isArray(classes) && classes.includes(name);
};

export function rehypeListingNames() {
	return (tree: Root) => {
		visit(tree, 'element', (node) => {
			if (node.tagName !== 'figure' || !hasClass(node, 'listing')) return;
			const caption = node.children.find(
				(child): child is Element => child.type === 'element' && child.tagName === 'figcaption',
			);
			const id = caption?.properties.id;
			if (typeof id !== 'string') return;
			visit(node, 'element', (inner) => {
				if (inner.tagName !== 'pre') return;
				inner.properties.role = 'group';
				inner.properties.ariaLabelledBy = [id];
			});
		});
	};
}

export function rehypeNamedPreBlocks() {
	return (tree: Root) => {
		let count = 0;
		visit(tree, 'element', (node) => {
			if (node.tagName !== 'pre' || hasClass(node, 'diagram__source')) return;
			if (node.properties.ariaLabelledBy || node.properties.ariaLabel) return;
			count += 1;
			node.properties.role = 'group';
			node.properties.ariaLabel = `Code listing ${count}`;
		});
	};
}

export function rehypeDiagrams() {
	return (tree: Root, file: VFile) => {
		let diagrams = 0;
		visit(tree, 'element', (node, index, parent) => {
			if (node.tagName !== 'pre' || !parent || index === undefined) return;
			const code = node.children.find(
				(child): child is Element => child.type === 'element' && child.tagName === 'code',
			);
			if (!code || !hasClass(code, 'language-mermaid')) return;
			const source = toString(code).replace(/\n+$/, '');
			const title = /^[ \t]*accTitle[ \t]*:[ \t]*(\S.*?)[ \t]*$/m.exec(source)?.[1];
			if (title === undefined || !/^[ \t]*accDescr[ \t]*(:[ \t]*\S|\{\s*[^\s}])/m.test(source)) {
				const where = `${file.data.repositoryPath ?? 'a chapter'}, line ${node.position?.start.line ?? '?'}`;
				throw new Error(
					`${where}: a diagram needs an accTitle and an accDescr, which name and describe it for a screen reader.`,
				);
			}
			diagrams += 1;
			const caption = `diagram-${diagrams}`;
			const figure: Element = {
				type: 'element',
				tagName: 'figure',
				properties: { className: ['diagram'] },
				children: [
					{
						type: 'element',
						tagName: 'div',
						properties: { className: ['diagram__scroll'] },
						children: [
							{
								type: 'element',
								tagName: 'pre',
								properties: { className: ['diagram__source'] },
								children: [{ type: 'text', value: source }],
							},
						],
					},
					{
						type: 'element',
						tagName: 'figcaption',
						properties: { id: caption, className: ['diagram__caption'] },
						children: [{ type: 'text', value: `Diagram: ${title}` }],
					},
				],
			};
			parent.children[index] = figure;
			return index + 1;
		});
	};
}

export function rehypeTables() {
	return (tree: Root) => {
		visit(tree, 'element', (node, index, parent) => {
			if (node.tagName !== 'table' || !parent || index === undefined) return;
			const headings: string[] = [];
			visit(node, 'element', (cell) => {
				if (cell.tagName === 'th') headings.push(toString(cell).trim());
			});
			parent.children[index] = {
				type: 'element',
				tagName: 'div',
				properties: {
					className: ['table-scroll'],
					role: 'group',
					tabIndex: 0,
					ariaLabel: headings.length > 0 ? `Table: ${headings.join(', ')}` : 'Table',
				},
				children: [node],
			};
			return index + 1;
		});
	};
}
