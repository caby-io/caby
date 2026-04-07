import { ApiRequestBuilder, type ApiClient, type ApiResponse, type Token } from './client';

export type LoginResp = {
	user: string;
	login_token: Token;
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
