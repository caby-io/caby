import type { Entry } from '$lib/files/entry';

export const SseKind = {
	ENTRY_CREATED: 'entry.created',
	ENTRY_MODIFIED: 'entry.modified',
	ENTRY_REMOVED: 'entry.removed',
	ENTRY_MOVED: 'entry.moved',
	STREAM_CONNECTED: 'stream.connected',
	STREAM_RESET: 'stream.reset'
} as const;
export type SseKind = (typeof SseKind)[keyof typeof SseKind];

export const SubscriptionKind = {
	DIR: 'dir',
	TREE: 'tree',
	SHARES: 'shares'
} as const;
export type SubscriptionKind = (typeof SubscriptionKind)[keyof typeof SubscriptionKind];

export type EntryChange = {
	space: string;
	path: string;
	from?: string;
	entry?: Entry;
};

export type SseEntryEvent = {
	kind:
		| typeof SseKind.ENTRY_CREATED
		| typeof SseKind.ENTRY_MODIFIED
		| typeof SseKind.ENTRY_REMOVED
		| typeof SseKind.ENTRY_MOVED;
	id: number;
	actor?: string;
	data: EntryChange;
	subscriptions: SubscriptionKind[];
};

export type SseStreamConnected = {
	kind: typeof SseKind.STREAM_CONNECTED;
	data: { connection_id: string };
};

export type SseStreamReset = {
	kind: typeof SseKind.STREAM_RESET;
};

export type SseEvent = SseEntryEvent | SseStreamConnected | SseStreamReset;
