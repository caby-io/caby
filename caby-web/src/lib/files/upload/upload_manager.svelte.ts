import xxhash from 'xxhash-wasm';
import { CABY_CHUNK_INDEX, CABY_UPLOAD_TOKEN, type UploadFileRef } from './upload';
import { TaskStatus, type UploadFile } from './upload_file.svelte';
import { UploadGroup, type UploadRegistration } from './upload_group';
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
	publishUpload,
	ConflictStrategy,
	stageUpload,
	putChunk,
	registerUpload
} from '$lib/api/api_files';

const MAX_REGISTER_THREADS = 3;
const MAX_HASH_THREADS = 3;
export const MAX_UPLOAD_THREADS = 3;

type UploadGroupCb = (upload_group: UploadGroup) => void;
type UploadFileCb = (ref: UploadFileRef) => void;

// todo: update the total
// todo: we may want to eventually batch registrations
const startRegisterFileWorker = async (on_done: UploadGroupCb, upload_group: UploadGroup) => {
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

const startHashFileWorker = async (on_done: UploadFileCb, ref: UploadFileRef) => {
	const [upload_group, upload_file] = ref;
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
	on_done(ref);
};

const startUploadFileWorker = async (
	on_done: UploadFileCb,
	ref: UploadFileRef,
	combined_progress: CombinedProgress
) => {
	const [upload_group, upload_file] = ref;
	const upload_id = combined_progress.registerUpload();

	let index = 0;

	// create a reader that pushes, at most, the chunk limit to the uploader
	const reader = new FileReader();
	const chunk_size = upload_group.registration!.chunk_size!;
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
			on_done(ref);
			console.debug('[caby/upload-manager] finished uploading chunks');
			return;
		}

		// todo: handle
		const resp = await putChunk(
			client,
			upload_group.registration,
			upload_file.space,
			upload_file.name,
			index,
			event.target!.result
		);

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

const getStartUploadPayload = (ref: UploadFileRef): StartUploadPayload => {
	const [upload_group, upload_file] = ref;
	return {
		client_config: client.getConfig(),
		space: upload_file.space,
		base_path: upload_group.base_path,
		name: upload_file.name,
		file: upload_file.file,
		registration: upload_group.registration
	};
};

const startUploadFileWorkerBackground = async (
	on_done: UploadFileCb,
	ref: UploadFileRef,
	combined_progress: CombinedProgress
) => {
	const [upload_group, upload_file] = ref;
	const upload_id = combined_progress.registerUpload();
	const upload_worker = new UploadWorker();
	let start_upload_message: Message<StartUploadPayload> = {
		event: MessageType.StartUpload,
		payload: getStartUploadPayload(ref)
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
				upload_file.upload_task_status = TaskStatus.COMPLETE;
				combined_progress.unregisterUpload(upload_id);
				on_done(ref);
				console.debug('[caby/upload-manager/worker-upload] finished uploading chunks');
				upload_worker.terminate();
				break;
			default:
				// todo: wrap in err type
				console.error('[caby/upload-manager/worker-upload] unhandled message:', e.data);
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

	// queues
	register_queue: UploadGroup[] = [];
	hash_queue: UploadFileRef[] = [];
	upload_queue: UploadFileRef[] = [];

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

		// reset progress if it's completed
		if (this.upload_progress.progress === this.upload_progress.total) {
			console.debug('[caby/upload-manager] resetting upload progress');
			this.upload_progress.reset();
		}

		upload_groups.forEach((g) => {
			// update totals
			this.upload_progress.addTotal(
				g.upload_files.reduce((accumulator, f) => accumulator + f.file.size, 0)
			);

			// push to queues
			this.register_queue.push(g);
			g.upload_files.forEach((f) => {
				this.hash_queue.push([g, f]);
			});
		});

		this.startRegistering();
	};

	private startRegistering = () => {
		const on_done_callback: UploadGroupCb = (g: UploadGroup) => {
			this.register_worker_count--;
			g.upload_files.forEach((f) => {
				this.upload_queue.push([g, f]);
			});
			this.startRegistering();
			this.startHashing();
			this.startUploading();
		};

		while (this.register_worker_count < MAX_REGISTER_THREADS && this.register_queue.length > 0) {
			this.register_worker_count++;
			console.debug(
				`[caby/upload-manager] starting registration worker ${this.register_worker_count}`
			);

			const next_upload = this.register_queue.shift()!;
			next_upload.registration_task_status = TaskStatus.STARTED;
			startRegisterFileWorker(on_done_callback, next_upload);
		}
	};

	private startHashing = () => {
		const on_done_callback: UploadFileCb = (ref: UploadFileRef) => {
			this.hash_worker_count--;
			this.startHashing();
			this.stageUpload(ref);
		};

		while (this.hash_worker_count < MAX_HASH_THREADS && this.hash_queue.length > 0) {
			this.hash_worker_count++;
			console.debug(`[caby/upload-manager] starting hashing worker ${this.hash_worker_count}`);

			const ref = this.hash_queue.shift()!;
			const [group, next_upload] = ref;
			next_upload.hash_task_status = TaskStatus.STARTED;
			startHashFileWorker(on_done_callback, ref);
		}
	};

	private startUploading = () => {
		const on_done_callback: UploadFileCb = (ref: UploadFileRef) => {
			// upload_file.upload_task_status = TaskStatus.COMPLETE;
			this.upload_worker_count--;
			this.startUploading();
			this.stageUpload(ref);
		};

		while (this.upload_worker_count < MAX_UPLOAD_THREADS && this.upload_queue.length > 0) {
			this.upload_worker_count++;
			console.debug(`[caby/upload-manager] starting upload worker ${this.upload_worker_count}`);

			const ref = this.upload_queue.shift()!;
			const [group, next_upload] = ref;
			next_upload.upload_task_status = TaskStatus.STARTED;
			if (window.Worker) {
				startUploadFileWorkerBackground(on_done_callback, ref, this.upload_progress);
				continue;
			}
			startUploadFileWorker(on_done_callback, ref, this.upload_progress);
		}
	};

	cancelUpload = (upload: UploadFile) => {
		// todo
	};

	private stageUpload = async (ref: UploadFileRef) => {
		const [upload_group, upload_file] = ref;
		if (!upload_file.readyToStage()) {
			return;
		}
		upload_file.stage_task_status = TaskStatus.STARTED;

		// todo: handle response and error
		const resp = await stageUpload(client, upload_group.registration, upload_file);

		upload_file.stage_task_status = TaskStatus.COMPLETE;
		console.debug('[caby/upload-manager] finished staging file');
		await this.tryPublishUploadGroup(upload_group);
	};

	private tryPublishUploadGroup = async (upload_group: UploadGroup) => {
		// todo: consider what to do for non-complete 'done' states
		// if even one file isn't ready then skip publishing
		if (upload_group?.upload_files.find((f) => f.stage_task_status !== TaskStatus.COMPLETE)) {
			return;
		}

		// todo: handle errors
		const resp = await publishUpload(client, upload_group!);

		console.debug(`[caby/upload-manager] completed ${upload_group.registration.id}`);

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
