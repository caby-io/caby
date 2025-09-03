import type { UploadFile } from './upload_file.svelte';
import type { UploadRegistration } from './upload_group';

export const CABY_UPLOAD_TOKEN = 'Caby-Upload-Token';
export const CABY_CHUNK_INDEX = 'Caby-Chunk-Index';

export enum EntryType {
	FILE = 'file',
	DIRECTORY = 'directory'
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

export type RegisterUploadRequest = {
	base_path: string;
	entries: UploadEntry[];
	conflict_strategy: ConflictStrategy;
};

export type Progress = {
	progress: number;
	total: number;
};

// todo: move to own file
// worker events
export enum EventType {
	UploadStart,
	UploadProgress,
	UploadCompleted
}

export type WorkerEvent<T> = {
	event: EventType;
	payload: T;
};

export type UploadStartPayload = {
	base_path: string;
	file: File;
	registration: UploadRegistration;
};

export type UploadProgressPayload = {
	new_progress: number;
};

export type UploadCompletePayload = {};
