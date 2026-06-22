export type SwipeOptions = {
	on_swipe: (dir: 'left' | 'right') => void;
	threshold?: number;
	max_duration_ms?: number;
};

// Svelte 5 action: commits a horizontal swipe on pointerup when:
//   - |dx| exceeds `threshold` (default 60px)
//   - the gesture is mostly horizontal (|dx| > 2 * |dy|)
//   - elapsed time is under `max_duration_ms` (default 500ms)
// Vertical scrolls / long presses / tiny wiggles do nothing.
export function swipe(node: HTMLElement, opts: SwipeOptions) {
	let options = opts;
	let start_x = 0;
	let start_y = 0;
	let start_t = 0;
	let tracking = false;

	const onPointerDown = (e: PointerEvent) => {
		if (!e.isPrimary || e.button !== 0) return;
		tracking = true;
		start_x = e.clientX;
		start_y = e.clientY;
		start_t = performance.now();
	};

	const onPointerUp = (e: PointerEvent) => {
		if (!tracking) return;
		tracking = false;
		const dx = e.clientX - start_x;
		const dy = e.clientY - start_y;
		const dt = performance.now() - start_t;
		const threshold = options.threshold ?? 60;
		const max_dt = options.max_duration_ms ?? 500;
		if (dt > max_dt) return;
		if (Math.abs(dx) < threshold) return;
		if (Math.abs(dx) < 2 * Math.abs(dy)) return;
		options.on_swipe(dx < 0 ? 'left' : 'right');
	};

	const onPointerCancel = () => {
		tracking = false;
	};

	node.addEventListener('pointerdown', onPointerDown);
	node.addEventListener('pointerup', onPointerUp);
	node.addEventListener('pointercancel', onPointerCancel);

	return {
		update(next: SwipeOptions) {
			options = next;
		},
		destroy() {
			node.removeEventListener('pointerdown', onPointerDown);
			node.removeEventListener('pointerup', onPointerUp);
			node.removeEventListener('pointercancel', onPointerCancel);
		}
	};
}
