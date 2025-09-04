import { EntryType, type UploadEntry as RegisterUploadEntry, type Progress } from './upload';
import type { UploadRegistration } from './upload_group';
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
	public base_path: string;
	public file: File;
	public registration: UploadRegistration;

	public hash_task_status: TaskStatus = TaskStatus.PENDING;
	public upload_task_status: TaskStatus = TaskStatus.PENDING;
	public finalize_task_status: TaskStatus = TaskStatus.PENDING;

	// public registration?: UploadRegistration;
	public xxh_digest?: string;
	public upload_progress: Progress;

	constructor(base_path: string, registration: UploadRegistration, file: File) {
		this.base_path = base_path;
		this.file = file;
		this.registration = registration;
		this.upload_progress = { progress: 0, total: file.size };
	}

	public readyToHash = (): boolean => {
		return this.registration.id !== undefined && this.hash_task_status === TaskStatus.PENDING;
	};

	public readyToUpload = (): boolean => {
		return this.registration.id !== undefined && this.upload_task_status === TaskStatus.PENDING;
	};

	public readyToFinalize = (): boolean => {
		return (
			this.registration.id !== undefined &&
			this.hash_task_status === TaskStatus.COMPLETE &&
			this.upload_task_status === TaskStatus.COMPLETE &&
			this.finalize_task_status === TaskStatus.PENDING
		);
	};

	public intoUploadEntry = (): RegisterUploadEntry => {
		return {
			entry_type: EntryType.FILE,
			// name: this.file.webkitRelativePath || this.file.name,
			name: this.file.name,
			size: this.file.size,
			xxh_digest: this.xxh_digest!
		};
	};

	public intoStartUploadPayload = (): StartUploadPayload => {
		return {
			base_path: this.base_path,
			file: this.file,
			registration: this.registration
		};
	};
}
