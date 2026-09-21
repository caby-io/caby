import { browser } from '$app/environment';

export type ThemeMode = 'auto' | 'light' | 'dark';

const STORAGE_KEY = 'color_scheme';

const isMode = (value: unknown): value is ThemeMode =>
	value === 'auto' || value === 'light' || value === 'dark';

const preferred = (): 'light' | 'dark' =>
	window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';

const loadMode = (): ThemeMode => {
	try {
		const stored = localStorage.color_scheme;
		return isMode(stored) ? stored : 'auto';
	} catch {
		return 'auto';
	}
};

export const colorScheme = $state<{ mode: ThemeMode }>({ mode: 'auto' });

const apply = () => {
	const resolved = colorScheme.mode === 'auto' ? preferred() : colorScheme.mode;
	document.documentElement.setAttribute('data-theme', resolved);
};

export const setMode = (mode: ThemeMode) => {
	colorScheme.mode = mode;
	try {
		if (mode === 'auto') localStorage.removeItem(STORAGE_KEY);
		else localStorage.color_scheme = mode;
	} catch {}
	apply();
};

let initialized = false;

export const initColorScheme = () => {
	if (!browser || initialized) return;
	initialized = true;
	colorScheme.mode = loadMode();
	apply();
	window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
		if (colorScheme.mode === 'auto') apply();
	});
};
