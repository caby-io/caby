import { TaskStatus, UploadFile } from './upload_file.svelte';

export type UploadRegistration = {
	id: string;
	chunk_size: number;
	token: string;
};

export class UploadGroup {
	public space: string;
	public base_path: string;
	public upload_files: UploadFile[];
	public registration: UploadRegistration;

	public registration_task_status: TaskStatus = TaskStatus.PENDING;
	// public commit_task_status: TaskStatus = TaskStatus.PENDING;

	constructor(space: string, base_path: string, ...files: File[]) {
		this.space = space;
		this.base_path = base_path;
		this.registration = {} as UploadRegistration;
		this.upload_files = files.map((f) => new UploadFile(space, base_path, f));
	}
}
