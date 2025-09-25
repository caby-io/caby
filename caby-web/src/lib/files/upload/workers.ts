import type { UploadRegistration } from './upload_group';

export enum MessageType {
	StartUpload,
	UploadProgress,
	UploadCompleted
}

export type Message<T> = {
	event: MessageType;
	payload: T;
};

export type StartUploadPayload = {
	base_path: string;
	file: File;
	registration: UploadRegistration;
};

export type UploadProgressPayload = {
	new_progress: number;
};

export type UploadCompletePayload = {};
