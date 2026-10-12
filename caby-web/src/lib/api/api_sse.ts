import type { SubscriptionKind } from '$lib/sse/sse_event';
import { ApiRequestBuilder, type ApiClient, type ApiResponse } from './client';

export type SseTokenData = {
	token: string;
	expires_at: string;
};

export type SseSubscriptionSpec = {
	kind: SubscriptionKind;
	space: string;
	path?: string;
};

export type SseSubscribeData = {
	revision: number;
};

export const getSseToken = async (client: ApiClient): Promise<ApiResponse<SseTokenData>> => {
	const req = ApiRequestBuilder.post('sse/token').intoRequest();
	return await client.exec(req);
};

export const sseUrl = (client: ApiClient, token: string): string => {
	return `${client.api_base}/sse?token=${encodeURIComponent(token)}`;
};

export const putSseSubscription = async (
	client: ApiClient,
	connection_id: string,
	spec: SseSubscriptionSpec
): Promise<ApiResponse<SseSubscribeData>> => {
	const req = ApiRequestBuilder.put(`sse/${connection_id}/subscriptions/${spec.kind}`)
		.withJsonBody({ space: spec.space, path: spec.path })
		.intoRequest();
	return await client.exec(req);
};

export const deleteSseSubscription = async (
	client: ApiClient,
	connection_id: string,
	kind: SubscriptionKind
): Promise<ApiResponse<string>> => {
	const req = ApiRequestBuilder.delete(`sse/${connection_id}/subscriptions/${kind}`).intoRequest();
	return await client.exec(req);
};
