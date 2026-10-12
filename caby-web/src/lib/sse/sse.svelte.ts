import {
	deleteSseSubscription,
	getSseToken,
	putSseSubscription,
	sseUrl,
	type SseSubscriptionSpec
} from '$lib/api/api_sse';
import { ApiStatus, type ApiClient } from '$lib/api/client';
import { SseKind, type SseEntryEvent, type SseEvent, type SubscriptionKind } from './sse_event';

const RETRY_MS = 3000;
const RETRY_JITTER_MS = 1000;

export const sse = $state({ connected: false });

type ActiveSubscription = {
	client: ApiClient;
	spec: SseSubscriptionSpec;
	handler: (ev: SseEntryEvent | null) => void;
};

let connection_id: string | null = null;
const subscriptions = new Map<SubscriptionKind, ActiveSubscription>();

let ops: Promise<void> = Promise.resolve();
const enqueue = (op: () => Promise<void>) => {
	ops = ops.then(op, op);
};

const trySubscribe = async (active: ActiveSubscription) => {
	if (subscriptions.get(active.spec.kind) !== active) return;
	if (!connection_id) {
		active.handler(null);
		return;
	}

	const resp = await putSseSubscription(active.client, connection_id, active.spec);
	if (subscriptions.get(active.spec.kind) !== active) return;
	if (resp.status !== ApiStatus.StatusSuccess) {
		console.error(`[caby/sse] could not subscribe to ${active.spec.kind}: ${resp.message}`);
	}
	active.handler(null);
};

const tryUnsubscribe = async (client: ApiClient, kind: SubscriptionKind) => {
	if (subscriptions.has(kind) || !connection_id) return;
	await deleteSseSubscription(client, connection_id, kind);
};

export const onSubscription = (
	client: ApiClient,
	spec: SseSubscriptionSpec,
	handler: (ev: SseEntryEvent | null) => void
): (() => void) => {
	const active = { client, spec, handler };
	subscriptions.set(spec.kind, active);
	enqueue(() => trySubscribe(active));

	return () => {
		if (subscriptions.get(spec.kind) !== active) return;
		subscriptions.delete(spec.kind);
		enqueue(() => tryUnsubscribe(client, spec.kind));
	};
};

// todo: use to handle event blocks
export const debounce = (fn: () => void, ms = 100): (() => void) => {
	let timer: ReturnType<typeof setTimeout> | undefined;
	return () => {
		clearTimeout(timer);
		timer = setTimeout(fn, ms);
	};
};

const dispatch = (ev: SseEvent) => {
	switch (ev.kind) {
		case SseKind.STREAM_CONNECTED:
			connection_id = ev.data.connection_id;
			sse.connected = true;
			for (const active of subscriptions.values()) {
				enqueue(() => trySubscribe(active));
			}
			return;
		case SseKind.STREAM_RESET:
			for (const active of subscriptions.values()) {
				active.handler(null);
			}
			return;
		default:
			for (const kind of ev.subscriptions) {
				subscriptions.get(kind)?.handler(ev);
			}
	}
};

const tryParse = (msg: MessageEvent<string>): SseEvent | null => {
	try {
		const ev = JSON.parse(msg.data) as SseEvent;
		if (ev.kind === SseKind.STREAM_CONNECTED || ev.kind === SseKind.STREAM_RESET) return ev;
		return { ...ev, id: Number(msg.lastEventId), subscriptions: ev.subscriptions ?? [] };
	} catch (err) {
		console.error(`[caby/sse] could not parse event: ${err}`);
		return null;
	}
};

export const tryConnectSse = (client: ApiClient): (() => void) => {
	let source: EventSource | null = null;
	let retry: ReturnType<typeof setTimeout> | undefined;
	let closed = false;

	const disconnect = () => {
		source?.close();
		source = null;
		connection_id = null;
		sse.connected = false;
	};

	const scheduleRetry = () => {
		// todo: exponential backoff
		retry = setTimeout(open, RETRY_MS + Math.random() * RETRY_JITTER_MS);
	};

	const open = async () => {
		// todo: cache the sse token across reconnects
		const resp = await getSseToken(client);
		if (closed) return;
		if (resp.status !== ApiStatus.StatusSuccess || !resp.data) {
			scheduleRetry();
			return;
		}

		source = new EventSource(sseUrl(client, resp.data.token));
		source.onmessage = (msg) => {
			const ev = tryParse(msg);
			if (ev) dispatch(ev);
		};
		source.onerror = () => {
			connection_id = null;
			sse.connected = false;
			if (source?.readyState !== EventSource.CLOSED) return;
			disconnect();
			scheduleRetry();
		};
	};

	open();

	return () => {
		closed = true;
		clearTimeout(retry);
		disconnect();
	};
};
