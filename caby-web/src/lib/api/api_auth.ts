import { ApiRequestBuilder, type ApiClient, type ApiResponse } from './client';

export type LoginTokenResp = {
	value: string;
	created_at: string;
	expires_at: string;
};

export type LoginResp = {
	user: string;
	login_token: LoginTokenResp;
};

export const login = async (
	client: ApiClient,
	login: string,
	password: string
): Promise<ApiResponse<LoginResp>> => {
	const req = ApiRequestBuilder.post(`auth/login`)
		.withJsonBody({ login: login, password: password })
		.intoRequest();
	return await client.exec(req);
};
