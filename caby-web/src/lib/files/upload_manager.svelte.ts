import xxhash from 'xxhash-wasm';
import {
	CABY_CHUNK_INDEX,
	CABY_UPLOAD_TOKEN,
	ConflictStrategy,
	type RegisterUploadRequest,
	type Progress,
	UploadStatus,
	HashStatus
} from './upload';
import { TaskStatus, type UploadFile } from './upload_file.svelte';

const MAX_HASH_THREADS = 3;
const MAX_UPLOAD_THREADS = 3;

type OnWorkerDone = (upload_file: UploadFile) => void;

// todo: update the total
// todo: we may want to eventually batch registrations
const start_register_file_worker = async (on_done: OnWorkerDone, upload_file: UploadFile) => {
	let register_request: RegisterUploadRequest = {
		base_path: upload_file.base_path,
		entries: [upload_file.into_upload_entry()],
		conflict_strategy: ConflictStrategy.OVERRIDE // todo: make this a param
	};

	const response = await fetch('http://localhost:8080/v0/files/upload', {
		method: 'post',
		headers: {
			Accept: 'application/json',
			'Content-Type': 'application/json'
		},
		body: JSON.stringify(register_request)
	});

	const response_payload = await response.json();
	// todo: check for error
	upload_file.registration = response_payload.data;

	console.debug(`upload manager: registered upload ${upload_file.registration?.id}`);
	upload_file.registration_task_status = TaskStatus.COMPLETE;
	on_done(upload_file);
};

const start_hash_file_worker = async (on_done: OnWorkerDone, upload_file: UploadFile) => {
	const reader = upload_file.file.stream().getReader();
	const { create64 } = await xxhash();

	const hasher = create64();
	while (true) {
		const { done, value } = await reader.read();
		if (done) {
			break;
		}
		hasher.update(value);
	}

	const digest = hasher.digest();
	upload_file.xxh_digest = digest.toString(16).padStart(16, '0');
	console.debug(`upload manager: xxh_digest: ${upload_file.xxh_digest}`);
	upload_file.hash_task_status = TaskStatus.COMPLETE;
	on_done(upload_file);
};

const start_upload_file_worker = async (
	on_done: OnWorkerDone,
	upload_file: UploadFile,
	global_progress: Progress
) => {
	const id = upload_file.registration!.id;
	// todo: better name?
	const name = upload_file.file.webkitRelativePath || upload_file.file.name;

	let index = 0;

	// create a reader that pushes, at most, the chunk limit to the uploader
	const reader = new FileReader();
	const chunk_size = upload_file.registration!.chunk_size!;
	const readNext = () => {
		const start = index * chunk_size;
		const end = start + chunk_size;
		// todo: should we detect completion here or before next readNext()?
		// console.debug(`start: ${start}, end: ${end}`);
		let slice = upload_file.file.slice(start, end);
		reader.readAsArrayBuffer(slice);
	};

	// because we are controlling the reader in readNext, we are guaranteed to have the bytes in order here
	reader.onload = async (event: ProgressEvent<FileReader>) => {
		const byte_length = (event.target!.result as ArrayBuffer).byteLength;
		// we are done
		if (byte_length < 1) {
			upload_file.upload_task_status = TaskStatus.COMPLETE;
			upload_file.upload_progress.progress = upload_file.upload_progress.total;
			on_done(upload_file);
			console.debug('upload manager: finished uploading chunks');
			return;
		}

		const response = await fetch(`http://localhost:8080/v0/files/upload/chunk/${id}/${name}`, {
			method: 'put',
			headers: {
				// todo: make these constants
				[CABY_UPLOAD_TOKEN]: upload_file.registration!.token,
				[CABY_CHUNK_INDEX]: index.toString()
			},
			body: event.target!.result
		});
		// todo: handle response and error

		// update file progress
		const last_progress = upload_file.upload_progress.progress;
		const start = index * chunk_size;
		const total_loaded = start + byte_length;
		upload_file.upload_progress.progress = total_loaded;

		// update total progress
		global_progress.progress += total_loaded - last_progress;

		index++;
		readNext();
	};

	// this shows some progress when the connection is slow
	// reader.onprogress = async (event: ProgressEvent<FileReader>) => {
	//     const start = index * chunk_size;
	//     const total_loaded = start + event.loaded
	//     console.log(total_loaded)
	//     calculate_progress(event.loaded)
	// }

	// start
	readNext();
};

// UploadManager runs upload requests thru the upload pipeline:
// 1. Register the upload request with the API
// 2. Hash the file and start uploading chunks
// 3. Notify the API when a file's upload is complete
// 4. Notify the API when an upload group is complete
export class UploadManager {
	uploads: UploadFile[] = $state([]);
	// todo: rename to upload progress?
	// todo: do we cache or calculate?
	progress: Progress = $state({ progress: 0, total: 0 });

	register_worker_count: number = 0;
	hash_worker_count: number = 0;
	upload_worker_count: number = 0;

	public addUploads = (...uploads: UploadFile[]) => {
		this.uploads.push(...uploads);
		// update totals
		uploads.forEach((u) => {
			this.progress.total += u.file.size;
		});
		this.startRegistering();
	};

	private startRegistering = () => {
		const on_done_callback: OnWorkerDone = (_upload_file: UploadFile) => {
			this.register_worker_count--;
			this.startRegistering();
			this.startHashing();
			this.startUploading();
		};

		let pending_registration = this.uploads.filter(
			(u) => u.registration_task_status === TaskStatus.PENDING
		);
		while (this.register_worker_count < MAX_HASH_THREADS && pending_registration.length > 0) {
			this.register_worker_count++;
			console.debug(`upload manager: starting registration worker ${this.register_worker_count}`);

			const next_upload = pending_registration.shift()!;
			next_upload.registration_task_status = TaskStatus.STARTED;
			start_register_file_worker(on_done_callback, next_upload);
		}
	};

	private startHashing = () => {
		const on_done_callback: OnWorkerDone = (upload_file: UploadFile) => {
			this.hash_worker_count--;
			this.startHashing();
			this.finalizeUpload(upload_file);
		};

		let pending_hashing = this.uploads.filter((u) => u.hash_task_status === TaskStatus.PENDING);
		while (this.hash_worker_count < MAX_HASH_THREADS && pending_hashing.length > 0) {
			this.hash_worker_count++;
			console.debug(`upload manager: starting hashing worker ${this.hash_worker_count}`);

			const next_upload = pending_hashing.shift()!;
			next_upload.hash_task_status = TaskStatus.STARTED;
			start_hash_file_worker(on_done_callback, next_upload);

			pending_hashing = this.uploads.filter((u) => u.hash_task_status === TaskStatus.PENDING);
		}
	};

	private startUploading = () => {
		const on_done_callback: OnWorkerDone = (upload_file: UploadFile) => {
			this.upload_worker_count--;
			this.startUploading();
			this.finalizeUpload(upload_file);
		};

		let pending_upload = this.uploads.filter((u) => u.upload_task_status === TaskStatus.PENDING);
		while (this.upload_worker_count < MAX_UPLOAD_THREADS && pending_upload.length > 0) {
			this.upload_worker_count++;
			console.debug(`upload manager: starting upload worker ${this.upload_worker_count}`);

			const next_upload = pending_upload.shift()!;
			next_upload.upload_task_status = TaskStatus.STARTED;
			start_upload_file_worker(on_done_callback, next_upload, this.progress);
		}
	};

	cancelUpload = (upload: UploadFile) => {
		// todo
	};

	private finalizeUpload = async (upload_file: UploadFile) => {
		if (
			upload_file.hash_task_status !== TaskStatus.COMPLETE ||
			upload_file.upload_task_status !== TaskStatus.COMPLETE ||
			upload_file.finalize_task_status !== TaskStatus.PENDING
		) {
			return;
		}
		upload_file.finalize_task_status = TaskStatus.STARTED;

		const id = upload_file.registration!.id;
		const name = upload_file.file.webkitRelativePath || upload_file.file.name;

		const response = await fetch(`http://localhost:8080/v0/files/upload/${id}/${name}`, {
			method: 'PATCH',
			headers: {
				[CABY_UPLOAD_TOKEN]: upload_file.registration!.token,
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				size: upload_file.file.size,
				xxh_digest: `${upload_file.xxh_digest}`,
				is_complete: true
			})
		});

		// todo: handle response and error

		upload_file.finalize_task_status = TaskStatus.COMPLETE;
		console.debug('upload manager: finished finalizing file');
		this.tryFinalizeUploadGroup(upload_file);
	};

	private tryFinalizeUploadGroup = async (upload_file: UploadFile) => {
		// todo: determine final name for 'upload group'
		let upload_id = upload_file.registration!.id;
		let upload_group = this.uploads.filter((u) => u.registration?.id === upload_id);
		// todo: consider what to do for non-complete 'done' states
		// if even one file isn't ready then skip finalizing

		if (
			upload_group.find(
				(u) =>
					u.upload_task_status !== TaskStatus.COMPLETE && u.hash_task_status !== TaskStatus.COMPLETE
			)
		) {
			console.debug('finalizer: group incomplete');
			return;
		}

		const response = await fetch(`http://localhost:8080/v0/files/upload/${upload_id}`, {
			method: 'post',
			headers: {
				[CABY_UPLOAD_TOKEN]: upload_file!.registration!.token
				// 'Content-Type': 'application/json'
			}
		});

		console.debug(`upload manager: completed ${upload_id}`);

		// all files for this group are uploaded

		// this.uploads.filter((u) => u.registration?.id === upload_id).forEach((u) => {
		//     // we have already encountered an upload from this group that wasn't uploaded
		//     if (groups.get(upload_id) === false) {
		//         return
		//     }

		//     break

		//     const upload_files = (groups.get(upload_id) || [])
		//     upload_files.push(u)
		//     groups.set(upload_id, upload_files)
		// })

		// this.uploads.filter((u) => u.state === UploadState.UPLOADED).forEach((u) => {
		//     if (!ready_cache.get(u.registration!.id)) {
		//         return
		//     }

		// })
	};
}

export const uploadManager = new UploadManager();
