import { parent } from '$lib/fs';
import { SseKind, type SseEntryEvent } from '$lib/sse/sse_event';
import { EntryType, type Entry } from './entry';

const trimSlashes = (path: string): string => path.replace(/^\/+|\/+$/g, '');

const basename = (path: string): string => path.substring(path.lastIndexOf('/') + 1);

const sortKey = (entry: Entry): string =>
	`${entry.entry_type === EntryType.DIRECTORY ? 0 : 1}_${entry.name}`;

const compareEntries = (a: Entry, b: Entry): number => {
	const a_key = sortKey(a);
	const b_key = sortKey(b);
	if (a_key < b_key) return -1;
	if (a_key > b_key) return 1;
	return 0;
};

export const patchEntries = (entries: Entry[], ev: SseEntryEvent, dir: string): Entry[] => {
	const current_dir = trimSlashes(dir);
	const in_dir = (path: string) => parent(path) === current_dir;
	let next = entries;

	if (ev.data.from !== undefined && in_dir(ev.data.from)) {
		const from_name = basename(ev.data.from);
		next = next.filter((e) => e.name !== from_name);
	}

	if (!in_dir(ev.data.path)) return next;

	const name = basename(ev.data.path);
	const without = next.filter((e) => e.name !== name);
	if (ev.kind === SseKind.ENTRY_REMOVED || !ev.data.entry) return without;

	return [...without, ev.data.entry].sort(compareEntries);
};
