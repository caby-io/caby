export const CABY_UPLOAD_TOKEN = 'Caby-Upload-Token';
export const CABY_CHUNK_INDEX = 'Caby-Chunk-Index';

// todo: move to api dir
export enum EntryType {
	FILE = 'file',
	DIRECTORY = 'directory'
}

// export type UploadEntry = {
// 	entry_type: string;
// 	name: string;
// 	size: number;
// 	xxh_digest: string;
// };

// export enum ConflictStrategy {
// 	OVERRIDE = 'override',
// 	SKIP = 'skip',
// 	PROMPT = 'prompt',
// 	DECONFLICT = 'deconflict'
// }

// export type RegisterUploadRequest = {
// 	base_path: string;
// 	entries: UploadEntry[];
// 	conflict_strategy: ConflictStrategy;
// };

// export type Progress = {

// };

type ProgressEvent = {
	time: number;
	progress: number;
};
