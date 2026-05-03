import { Progress } from './progress.svelte';
import { EntryType } from './upload';
import type { UploadEntry as RegisterUploadEntry } from '$lib/api/api_files';
import type { ClientConfig } from '$lib/api/client';
import type { UploadGroup, UploadRegistration } from './upload_group';
import type { StartUploadPayload } from './workers';

export enum TaskStatus {
	PENDING,
	STARTED,
	COMPLETE
}

// export enum RegistrationStatus {
//     PENDING,
//     REGISTERING,
//     COMPLETE
// }

// export enum HashStatus {
//     PENDING,
//     HASHING,
//     COMPLETE,
// }

// export enum UploadStatus {
//     PENDING,
//     UPLOADING,
//     COMPLETE
// }

export class UploadFile {
	public space: string;
	public base_path: string;
	public file: File;
	// public registration: UploadRegistration;

	public hash_task_status: TaskStatus = TaskStatus.PENDING;
	public upload_task_status: TaskStatus = TaskStatus.PENDING;
	public stage_task_status: TaskStatus = TaskStatus.PENDING;

	// public registration?: UploadRegistration;
	public xxh_digest?: string;
	public upload_progress: Progress;
	public upload_id?: number;

	constructor(space: string, base_path: string, file: File) {
		this.space = space;
		this.base_path = base_path;
		this.file = file;
		// this.registration = registration;
		this.upload_progress = new Progress(file.size);
	}

	public getCleanedName = (): string => {
		const name = this.file.webkitRelativePath || this.file.name;
		return name.split('/').map(encodeURIComponent).join('/');
	};

	// public readyToHash = (): boolean => {
	// 	return this.registration.id !== undefined && this.hash_task_status === TaskStatus.PENDING;
	// };

	// public readyToUpload = (): boolean => {
	// 	return this.registration.id !== undefined && this.upload_task_status === TaskStatus.PENDING;
	// };

	public readyToStage = (): boolean => {
		return (
			this.hash_task_status === TaskStatus.COMPLETE &&
			this.upload_task_status === TaskStatus.COMPLETE &&
			this.stage_task_status === TaskStatus.PENDING
		);
	};

	public intoUploadEntry = (): RegisterUploadEntry => {
		return {
			entry_type: EntryType.FILE,
			name: this.file.webkitRelativePath || this.file.name,
			size: this.file.size,
			xxh_digest: this.xxh_digest!
		};
	};
}
