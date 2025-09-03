import { TaskStatus, UploadFile } from './upload_file.svelte';

export type UploadRegistration = {
	id: string;
	chunk_size: number;
	token: string;
};

export class UploadGroup {
	public base_path: string;
	public upload_files: UploadFile[];
	public registration: UploadRegistration;

	public registration_task_status: TaskStatus = TaskStatus.PENDING;
	// public commit_task_status: TaskStatus = TaskStatus.PENDING;

	constructor(base_path: string, ...files: File[]) {
		this.base_path = base_path;
		this.registration = {} as UploadRegistration;
		this.upload_files = files.map((f) => new UploadFile(base_path, this.registration, f));
	}
}
