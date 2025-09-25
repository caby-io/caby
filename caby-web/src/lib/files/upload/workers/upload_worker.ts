// if (window.Worker) {
// 	onmessage = (e) => {
// 		console.log('Message received from main script');
// 		const workerResult = `Result: ${e.data[0] * e.data[1]}`;
// 		console.log('Posting message back to main script');
// 		postMessage(workerResult);
// 	};
// }

import { CABY_CHUNK_INDEX, CABY_UPLOAD_TOKEN } from '../upload';
import {
	MessageType,
	type Message,
	type StartUploadPayload,
	type UploadCompletePayload,
	type UploadProgressPayload
} from '../workers';

// todo: handle more messages
self.onmessage = function (e: MessageEvent<Message<any>>) {
	switch (e.data?.event) {
		case MessageType.StartUpload:
			const payload = e.data!.payload as StartUploadPayload;
			start_upload(payload);
			break;
		default:
			// todo: wrap in err type
			self.postMessage('unhandled err');
	}
};

let progress = 0;

const start_upload = (payload: StartUploadPayload) => {
	const id = payload.registration!.id;
	// todo: better name?
	const name = payload.file.name;

	let index = 0;

	// create a reader that pushes, at most, the chunk limit to the uploader
	const reader = new FileReader();
	const chunk_size = payload.registration!.chunk_size!;
	const readNext = () => {
		const start = index * chunk_size;
		const end = start + chunk_size;
		// todo: should we detect completion here or before next readNext()?
		// console.debug(`start: ${start}, end: ${end}`);
		let slice = payload.file.slice(start, end);
		reader.readAsArrayBuffer(slice);
	};

	// because we are controlling the reader in readNext, we are guaranteed to have the bytes in order here
	reader.onload = async (event: ProgressEvent<FileReader>) => {
		const byte_length = (event.target!.result as ArrayBuffer).byteLength;
		// we are done
		if (byte_length < 1) {
			const completedEvent: Message<UploadCompletePayload> = {
				event: MessageType.UploadCompleted,
				payload: {}
			};
			self.postMessage(completedEvent);
			// upload_file.upload_task_status = TaskStatus.COMPLETE;
			// upload_file.upload_progress.progress = upload_file.upload_progress.total;
			// on_done(upload_file);
			// console.debug('[caby/upload-manager] finished uploading chunks');
			return;
		}

		const response = await fetch(`http://localhost:8080/v0/files/upload/chunk/${id}/${name}`, {
			method: 'put',
			headers: {
				// todo: make these constants
				[CABY_UPLOAD_TOKEN]: payload.registration!.token,
				[CABY_CHUNK_INDEX]: index.toString()
			},
			body: event.target!.result
		});
		// todo: handle response and error

		// todo: store this in worker scope to make it clear that we have no access to the original upload_file obj
		// update file progress
		// const last_progress = progress;
		// const start = index * chunk_size;
		// const total_loaded = start + byte_length;
		// progress = total_loaded;

		// update total progress
		const progressEvent: Message<UploadProgressPayload> = {
			event: MessageType.UploadProgress,
			payload: { new_progress: byte_length }
		};
		self.postMessage(progressEvent);

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
