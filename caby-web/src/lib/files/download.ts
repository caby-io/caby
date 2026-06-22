import { getDownloadURL } from '$lib/api/api_files';
import type { ApiClient } from '$lib/api/client';
import type { Entry } from './entry';

export type DownloadToken = {
	value: string;
	space: string;
	file_paths: Array<string>;
	issued_at: string;
	expires_at: string;
};

export async function downloadEntries(client: ApiClient, space: string, entries: Entry[]) {
	if (entries.length === 0) return;
	const url = await getDownloadURL(client, space, entries);
	if (!url) {
		console.error('could not get download url');
		return;
	}
	const a = document.createElement('a');
	a.href = url;
	a.download = '';
	document.body.appendChild(a);
	a.click();
	a.remove();
}
