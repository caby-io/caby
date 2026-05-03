import type { ClientConfig } from '$lib/api/client';
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
	client_config: ClientConfig;
	space: string;
	base_path: string;
	name: string;
	file: File;
	registration: UploadRegistration;
};

export type UploadProgressPayload = {
	new_progress: number;
};

export type UploadCompletePayload = {};
