import { PutEntryType } from '$lib/files/api';
import type { Entry } from '$lib/files/entry';
import type { OverviewEntry } from '$lib/files/overview/overview_entry';
import { CABY_CHUNK_INDEX, CABY_UPLOAD_TOKEN } from '$lib/files/upload/upload';
import type { UploadFile } from '$lib/files/upload/upload_file.svelte';
import type { UploadGroup } from '$lib/files/upload/upload_group';
import type { StartUploadPayload } from '$lib/files/upload/workers';
import { join } from '$lib/fs';
import { ApiClient, ApiRequestBuilder, type ApiResponse } from './client';

export type ListFilesResp = {
	path: string | null;
	parent_dir: string | null;
	current_dir: string;
	entries: Array<Entry>;
};

export const listFiles = async (
	client: ApiClient,
	space: string,
	path: string
): Promise<ApiResponse<ListFilesResp>> => {
	const req = ApiRequestBuilder.get(`files/list/${space}/${path}`).intoRequest();
	return await client.exec(req);
};

export type FilesOverviewResp = {
	path: string | null;
	parent_dir: string | null;
	current_dir: string;
	entries: Array<OverviewEntry>;
};

export const filesOverview = async (
	client: ApiClient,
	space: string,
	path: string,
	dirs_only: boolean = false,
): Promise<ApiResponse<FilesOverviewResp>> => {
	const req = ApiRequestBuilder.get(`files/overview/${space}/${path}${dirs_only ? `?dirs_only=${dirs_only}` : ''}`).intoRequest();
	return await client.exec(req);
};

export type Move = [string, string];

export type MoveFilesResponse = {
	moved: Array<[string, string]>;
	errors: Array<any>; // todo
};

export const moveFiles = async (
	client: ApiClient,
	src_space: string,
	entries: Array<Move>,
	force: boolean = false,
	dst_space?: string
): Promise<ApiResponse<MoveFilesResponse>> => {
	// todo: handle different destination
	const req = ApiRequestBuilder.post(`files/move/${src_space}`)
		.withJsonBody({ entries, force })
		.intoRequest();
	return await client.exec(req);
};

export const getDownloadURL = (client: ApiClient, space: string, entries: Array<Entry>): string => {
	if (entries.length > 1) {
		// todo
		console.error('multi-download not implemented');
		return '';
	}

	return join(
		client.api_base,
		'files/download',
		encodeURIComponent(space),
		encodeURIComponent(entries[0].path)
	);
};

export const putEntry = async (client: ApiClient, space: string, path: string, entry_type: PutEntryType, name: string, content?: any) => {
	const req = ApiRequestBuilder.put(`files/${space}/${path}`)
		.withJsonBody({
			entry_type,
			name,
			...(content) && { content }
		})
		.intoRequest();
	return await client.exec(req);
}

export type UploadEntry = {
	entry_type: string;
	name: string;
	size: number;
	xxh_digest: string;
};

export enum ConflictStrategy {
	OVERRIDE = 'override',
	SKIP = 'skip',
	PROMPT = 'prompt',
	DECONFLICT = 'deconflict'
}

// todo: response type
export const registerUpload = async (
	client: ApiClient,
	space: string,
	path: string,
	entries: Array<UploadEntry>,
	conflict_strategy: ConflictStrategy = ConflictStrategy.OVERRIDE
) => {
	const req = ApiRequestBuilder.post(`files/upload/${space}`)
		.withJsonBody({
			base_path: path,
			entries,
			conflict_strategy
		})
		.intoRequest();
	return await client.exec(req);
};

export const putChunk = async (
	client: ApiClient,
	upload_file: UploadFile | StartUploadPayload,
	chunk_index: number,
	chunk: any
) => {
	const space = upload_file.space;
	const id = upload_file.registration!.id;
	const name = upload_file.file.webkitRelativePath || upload_file.file.name;
	const req = ApiRequestBuilder.put(`files/upload/${space}/chunk/${id}/${encodeURIComponent(name)}`)
		.addHeaders({
			[CABY_UPLOAD_TOKEN]: upload_file.registration!.token,
			[CABY_CHUNK_INDEX]: chunk_index.toString()
		})
		.withBody(chunk)
		.intoRequest();
	return await client.exec(req);
};

export const finalizeUpload = async (client: ApiClient, upload_file: UploadFile) => {
	const space = upload_file.space;
	const id = upload_file.registration!.id;
	const name = upload_file.file.webkitRelativePath || upload_file.file.name;
	const req = ApiRequestBuilder.patch(`files/upload/${space}/${id}/${encodeURIComponent(name)}`)
		.addHeaders({
			[CABY_UPLOAD_TOKEN]: upload_file.registration!.token
		})
		.withJsonBody({
			size: upload_file.file.size,
			xxh_digest: `${upload_file.xxh_digest}`,
			is_complete: true
		})
		.intoRequest();
	return await client.exec(req);
};

export const commitUpload = async (client: ApiClient, upload_group: UploadGroup) => {
	const space = upload_group.space;
	const id = upload_group.registration!.id;
	const req = ApiRequestBuilder.post(`files/upload/${space}/${id}`)
		.addHeaders({ [CABY_UPLOAD_TOKEN]: upload_group.registration!.token })
		.intoRequest();
	return await client.exec(req);
};

export const deleteFiles = async (
	client: ApiClient,
	space: string,
	entries: Array<Entry>,
	force: boolean = true
) => {
	const resp = ApiRequestBuilder.post(`files/delete/${space}`)
		.withJsonBody({
			entries: entries.map((e) => e.path),
			force
		})
		.intoRequest();
	return await client.exec(resp);
};
