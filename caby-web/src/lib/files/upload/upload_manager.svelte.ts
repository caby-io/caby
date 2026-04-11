import xxhash from 'xxhash-wasm';
import { CABY_CHUNK_INDEX, CABY_UPLOAD_TOKEN } from './upload';
import { TaskStatus, type UploadFile } from './upload_file.svelte';
import type { UploadGroup, UploadRegistration } from './upload_group';
import UploadWorker from './workers/upload_worker?worker';
import {
	MessageType,
	type UploadProgressPayload,
	type StartUploadPayload,
	type Message
} from './workers';
import { CombinedProgress, Progress } from './progress.svelte';
import { client } from '$lib/stores/client.svelte';
import {
	commitUpload,
	ConflictStrategy,
	finalizeUpload,
	putChunk,
	registerUpload
} from '$lib/api/api_files';

export const MAX_HASH_THREADS = 3;
export const MAX_UPLOAD_THREADS = 3;

type UploadGroupCb = (upload_group: UploadGroup) => void;
type UploadFileCb = (upload_file: UploadFile) => void;

// todo: update the total
// todo: we may want to eventually batch registrations
const startRegisterFileWorker = async (on_done: UploadGroupCb, upload_group: UploadGroup) => {
	// let register_request: RegisterUploadRequest = {
	// 	base_path: upload_group.base_path,
	// 	entries: [...upload_group.upload_files.map((f) => f.intoUploadEntry())],
	// 	conflict_strategy: ConflictStrategy.OVERRIDE // todo: make this a param
	// };

	const resp = await registerUpload(
		client,
		upload_group.space,
		upload_group.base_path,
		[...upload_group.upload_files.map((f) => f.intoUploadEntry())],
		ConflictStrategy.OVERRIDE
	);
	// todo: check for error
	Object.assign(upload_group.registration, resp.data as UploadRegistration);

	console.debug(`[caby/upload-manager] registered upload ${upload_group.registration!.id}`);
	upload_group.registration_task_status = TaskStatus.COMPLETE;
	on_done(upload_group);
};

const startHashFileWorker = async (on_done: UploadFileCb, upload_file: UploadFile) => {
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
	console.debug(`[caby/upload-manager] xxh_digest: ${upload_file.xxh_digest}`);
	upload_file.hash_task_status = TaskStatus.COMPLETE;
	on_done(upload_file);
};

const startUploadFileWorker = async (
	on_done: UploadFileCb,
	upload_file: UploadFile,
	combined_progress: CombinedProgress
) => {
	const id = upload_file.registration!.id;
	// todo: better name?
	const name = upload_file.file.webkitRelativePath || upload_file.file.name;
	const upload_id = combined_progress.registerUpload();

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
			// todo: remove this??
			// upload_file.upload_progress.progress = upload_file.upload_progress.total;
			combined_progress.unregisterUpload(upload_id);
			on_done(upload_file);
			console.debug('[caby/upload-manager] finished uploading chunks');
			return;
		}

		// todo: handle
		const resp = await putChunk(client, upload_file, index, event.target!.result);

		// update file progress
		// const last_progress = upload_file.upload_progress.progress;
		// const start = index * chunk_size;
		// const total_loaded = start + byte_length;
		// upload_file.upload_progress.setProgress(total_loaded);

		// update progress
		upload_file.upload_progress.addProgress(byte_length);
		combined_progress.addProgress(byte_length);
		combined_progress.setRate(upload_id, upload_file.upload_progress.rate);

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

const startUploadFileWorkerBackground = async (
	on_done: UploadFileCb,
	upload_file: UploadFile,
	combined_progress: CombinedProgress
) => {
	const upload_id = combined_progress.registerUpload();
	const upload_worker = new UploadWorker();
	let start_upload_message: Message<StartUploadPayload> = {
		event: MessageType.StartUpload,
		payload: upload_file.intoStartUploadPayload(client.auth.login_token!)
	};
	upload_worker.onmessage = function (e: MessageEvent<Message<any>>) {
		switch (e.data?.event) {
			case MessageType.UploadProgress:
				const payload = e.data!.payload as UploadProgressPayload;
				upload_file.upload_progress.addProgress(payload.new_progress);
				combined_progress.addProgress(payload.new_progress);
				combined_progress.setRate(upload_id, upload_file.upload_progress.rate);
				break;
			case MessageType.UploadCompleted:
				// upload_file.upload_task_status = TaskStatus.COMPLETE;
				combined_progress.unregisterUpload(upload_id);
				on_done(upload_file);
				console.debug('[caby/upload-manager/worker-upload] finished uploading chunks');
				// todo: cleanup worker?
				break;
			default:
				// todo: wrap in err type
				self.postMessage('unhandled err');
		}
	};
	upload_worker.postMessage(start_upload_message);
};
// UploadManager runs upload requests thru the upload pipeline:
// 1. Register the upload request with the API
// 2. Hash the file and start uploading chunks
// 3. Notify the API when a file's upload is complete
// 4. Notify the API when an upload group is complete
export class UploadManager {
	upload_groups: UploadGroup[] = $state([]);
	upload_files: UploadFile[] = $derived(this.upload_groups.flatMap((g) => [...g.upload_files]));

	// todo: rename to upload progress?
	// todo: do we cache or calculate?
	upload_progress: CombinedProgress = $state(new CombinedProgress());

	register_worker_count: number = 0;
	hash_worker_count: number = 0;
	upload_worker_count: number = 0;

	// TEMP
	upload_groups_completed: number = $state(0);

	public addUploads = (...upload_groups: UploadGroup[]) => {
		this.upload_groups.push(...upload_groups);

		// reset progress
		if (this.upload_progress.progress === this.upload_progress.total) {
			console.debug('[caby/upload-manager] resetting upload progress');
			this.upload_progress.reset();
		}

		// update totals
		upload_groups.forEach((g) => {
			this.upload_progress.addTotal(
				g.upload_files.reduce((accumulator, f) => accumulator + f.file.size, 0)
			);
		});

		this.startRegistering();
	};

	private startRegistering = () => {
		const on_done_callback: UploadGroupCb = (_?: UploadGroup) => {
			this.register_worker_count--;
			this.startRegistering();
			this.startHashing();
			this.startUploading();
		};

		// todo: this is expensive to be doing every time this fn is called. consider not having to do this each time
		let pending_registration = this.upload_groups.filter(
			(g) => g.registration_task_status === TaskStatus.PENDING
		);
		while (this.register_worker_count < MAX_HASH_THREADS && pending_registration.length > 0) {
			this.register_worker_count++;
			console.debug(
				`[caby/upload-manager] starting registration worker ${this.register_worker_count}`
			);

			const next_upload = pending_registration.shift()!;
			next_upload.registration_task_status = TaskStatus.STARTED;
			startRegisterFileWorker(on_done_callback, next_upload);
		}
	};

	private startHashing = () => {
		const on_done_callback: UploadFileCb = (upload_file: UploadFile) => {
			this.hash_worker_count--;
			this.startHashing();
			this.finalizeUpload(upload_file);
		};

		let pending_hashing = this.upload_files.filter((f) => f.readyToHash());
		while (this.hash_worker_count < MAX_HASH_THREADS && pending_hashing.length > 0) {
			this.hash_worker_count++;
			console.debug(`[caby/upload-manager] starting hashing worker ${this.hash_worker_count}`);

			const next_upload = pending_hashing.shift()!;
			next_upload.hash_task_status = TaskStatus.STARTED;
			startHashFileWorker(on_done_callback, next_upload);

			pending_hashing = this.upload_files.filter((u) => u.hash_task_status === TaskStatus.PENDING);
		}
	};

	private startUploading = () => {
		const on_done_callback: UploadFileCb = (upload_file: UploadFile) => {
			upload_file.upload_task_status = TaskStatus.COMPLETE;
			this.upload_worker_count--;
			this.startUploading();
			this.finalizeUpload(upload_file);
		};

		let pending_uploads = this.upload_files.filter((u) => u.readyToUpload());
		while (this.upload_worker_count < MAX_UPLOAD_THREADS && pending_uploads.length > 0) {
			this.upload_worker_count++;
			console.debug(`[caby/upload-manager] starting upload worker ${this.upload_worker_count}`);

			const next_upload = pending_uploads.shift()!;
			next_upload.upload_task_status = TaskStatus.STARTED;
			if (window.Worker) {
				startUploadFileWorkerBackground(on_done_callback, next_upload, this.upload_progress);
				continue;
			}
			startUploadFileWorker(on_done_callback, next_upload, this.upload_progress);
		}
	};

	cancelUpload = (upload: UploadFile) => {
		// todo
	};

	private finalizeUpload = async (upload_file: UploadFile) => {
		if (!upload_file.readyToFinalize()) {
			return;
		}
		upload_file.finalize_task_status = TaskStatus.STARTED;

		// todo: handle response and error
		const resp = await finalizeUpload(client, upload_file);

		upload_file.finalize_task_status = TaskStatus.COMPLETE;
		console.debug('[caby/upload-manager] finished finalizing file');
		await this.tryCommitUploadGroup(upload_file);
	};

	private tryCommitUploadGroup = async (upload_file: UploadFile) => {
		// todo: determine final name for 'upload group'
		let upload_id = upload_file.registration!.id;
		let upload_group = this.upload_groups.find((u) => u.registration?.id === upload_id);
		// todo: consider what to do for non-complete 'done' states
		// if even one file isn't ready then skip comitting
		if (upload_group?.upload_files.find((f) => f.finalize_task_status !== TaskStatus.COMPLETE)) {
			return;
		}

		// todo: handle errors
		const resp = await commitUpload(client, upload_group!);

		console.debug(`[caby/upload-manager] completed ${upload_id}`);

		// TEMP
		this.upload_groups_completed = Date.now();
	};
}

export const uploadManager = new UploadManager();

// todo
// window.addEventListener('beforeunload', (event) => {
// 	// Cancel the event as stated by the standard.
// 	event.preventDefault();
// 	// Chrome requires returnValue to be set.
// 	event.returnValue = 'test';
// });
