import type { Space } from '$lib/space';
import { ApiRequestBuilder, ApiClient, type ApiResponse } from './client';

export type ListSpacesResp = Array<Space>;

export const getSpaces = async (client: ApiClient): Promise<ApiResponse<ListSpacesResp>> => {
	const req = ApiRequestBuilder.get(`spaces`).intoRequest();
	return await client.exec(req);
};
