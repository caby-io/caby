import { PutEntryType } from '$lib/files/api';
import type { DownloadToken } from '$lib/files/download';
import type { Entry } from '$lib/files/entry';
import type { OverviewEntry } from '$lib/files/overview/overview_entry';
import { CABY_CHUNK_INDEX, CABY_UPLOAD_TOKEN } from '$lib/files/upload/upload';
import type { UploadFile } from '$lib/files/upload/upload_file.svelte';
import type { UploadGroup, UploadRegistration } from '$lib/files/upload/upload_group';
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

export const getFilesOverview = async (
	client: ApiClient,
	space: string,
	path: string,
	dirs_only: boolean = false
): Promise<ApiResponse<FilesOverviewResp>> => {
	const req = ApiRequestBuilder.get(
		`files/overview/${space}/${path}${dirs_only ? `?dirs_only=${dirs_only}` : ''}`
	).intoRequest();
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
	moves: Array<Move>,
	force: boolean = false,
	dst_space?: string
): Promise<ApiResponse<MoveFilesResponse>> => {
	// todo: handle different destination
	const req = ApiRequestBuilder.post(`files/move/${src_space}`)
		.withJsonBody({ entries: moves, force })
		.intoRequest();
	return await client.exec(req);
};

export type DownloadTokenResp = {
	token: DownloadToken;
};

export const getDownloadToken = async (
	client: ApiClient,
	space: string,
	entries: Array<Entry>
): Promise<ApiResponse<DownloadTokenResp>> => {
	const req = ApiRequestBuilder.post(`files/download/${space}`)
		.withJsonBody({
			files: entries.map((e) => e.path)
		})
		.intoRequest();

	return await client.exec(req);
};

export const getDownloadURL = async (
	client: ApiClient,
	space: string,
	entries: Array<Entry>
): Promise<string | undefined> => {
	if (entries.length > 1) {
		// todo
		console.error('multi-download not implemented');
		return '';
	}

	let resp = await getDownloadToken(client, space, entries);
	if (resp.status != 'success' || !resp.data) {
		console.error(`could not get download token: ${resp.message}`);
		return;
	}

	const url = join(
		client.api_base,
		'files/download',
		encodeURIComponent(space),
		encodeURIComponent(entries[0].path)
	);
	return `${url}?token=${encodeURIComponent(resp.data.token.value)}`;
};

export const putEntry = async (
	client: ApiClient,
	space: string,
	path: string,
	entry_type: PutEntryType,
	name: string,
	content?: any
) => {
	const req = ApiRequestBuilder.put(`files/${space}/${path}`)
		.withJsonBody({
			entry_type,
			name,
			...(content && { content })
		})
		.intoRequest();
	return await client.exec(req);
};

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
// todo: for now we've disabled redirects on all upload operations but consider only doing this for
export const registerUpload = async (
	client: ApiClient,
	space: string,
	path: string,
	entries: Array<UploadEntry>,
	conflict_strategy: ConflictStrategy = ConflictStrategy.OVERRIDE
) => {
	const req = ApiRequestBuilder.post(`files/upload/${space}`)
		.noRedirect()
		.withJsonBody({
			base_path: path,
			entries,
			conflict_strategy
		})
		.intoRequest();
	return await client.exec(req);
};

const encodePath = (name: string): string => {
	return name.split('/').map(encodeURIComponent).join('/');
};

export const putChunk = async (
	client: ApiClient,
	registration: UploadRegistration,
	space: string,
	name: string,
	chunk_index: number,
	chunk: any
) => {
	const id = registration.id;
	const req = ApiRequestBuilder.put(`files/upload/${space}/chunk/${id}/${encodePath(name)}`)
		.addHeaders({
			[CABY_UPLOAD_TOKEN]: registration.token,
			[CABY_CHUNK_INDEX]: chunk_index.toString()
		})
		.noRedirect()
		.withBody(chunk)
		.intoRequest();
	return await client.exec(req);
};

export const stageUpload = async (
	client: ApiClient,
	registration: UploadRegistration,
	upload_file: UploadFile
) => {
	const id = registration.id;
	const req = ApiRequestBuilder.patch(
		`files/upload/${upload_file.space}/${id}/${encodePath(upload_file.name)}`
	)
		.addHeaders({
			[CABY_UPLOAD_TOKEN]: registration.token
		})
		.noRedirect()
		.withJsonBody({
			size: upload_file.file.size,
			xxh_digest: `${upload_file.xxh_digest}`,
			is_complete: true
		})
		.intoRequest();
	return await client.exec(req);
};

export const publishUpload = async (client: ApiClient, upload_group: UploadGroup) => {
	const space = upload_group.space;
	const id = upload_group.registration!.id;
	const req = ApiRequestBuilder.post(`files/upload/${space}/${id}`)
		.noRedirect()
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
