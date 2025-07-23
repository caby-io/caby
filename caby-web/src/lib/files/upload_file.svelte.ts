import {
	EntryType,
	type UploadEntry as RegisterUploadEntry,
	type Progress,
	type UploadRegistration,
	HashStatus,
	UploadStatus
} from './upload';

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
	public base_path: string;
	public file: File;

	public registration_task_status: TaskStatus = TaskStatus.PENDING;
	public hash_task_status: TaskStatus = TaskStatus.PENDING;
	public upload_task_status: TaskStatus = TaskStatus.PENDING;
	public finalize_task_status: TaskStatus = TaskStatus.PENDING;

	public registration: UploadRegistration | null = null;
	public xxh_digest: string | null = null;
	public upload_progress: Progress;

	constructor(base_path: string, file: File) {
		this.base_path = base_path;
		this.file = file;
		this.upload_progress = { progress: 0, total: file.size };
	}

	public into_upload_entry = (): RegisterUploadEntry => {
		return {
			entry_type: EntryType.FILE,
			name: this.file.webkitRelativePath || this.file.name,
			size: this.file.size,
			xxh_digest: this.xxh_digest!
		};
	};
}
