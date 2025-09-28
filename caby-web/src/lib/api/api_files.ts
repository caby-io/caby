import type { Entry } from '$lib/files/entry';

export type FilesResponse = {
	path: string | null;
	parent_dir: string | null;
	current_dir: string;
	entries: Array<Entry>;
};
