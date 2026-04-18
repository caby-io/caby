// if (window.Worker) {
// 	onmessage = (e) => {
// 		console.log('Message received from main script');
// 		const workerResult = `Result: ${e.data[0] * e.data[1]}`;
// 		console.log('Posting message back to main script');
// 		postMessage(workerResult);
// 	};
// }

import { putChunk } from '$lib/api/api_files';
import { ApiClient } from '$lib/api/client';
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
			start_upload(new ApiClient(payload.client_config), payload);
			break;
		default:
			// todo: wrap in err type
			self.postMessage('unhandled err');
	}
};

const start_upload = (client: ApiClient, payload: StartUploadPayload) => {
	// const id = payload.registration!.id;
	// todo: better name?
	// const name = payload.file.name;

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
			return;
		}
		const resp = await putChunk(client, payload, index, event.target!.result);
		// todo: handle response and error

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
