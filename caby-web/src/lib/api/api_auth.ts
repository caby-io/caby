import { ApiRequestBuilder, type ApiClient, type Token } from './client';

export type LoginResp = {
	user: string;
	login_token: Token;
};

type LoginSuccess = { status: 'success'; status_code: number; data: LoginResp };
type LoginFail = { status: 'fail'; status_code: number; data: string };
type LoginError = { status: 'error'; status_code: number; message: string };

export type LoginResult = LoginSuccess | LoginFail | LoginError;

export const login = async (
	client: ApiClient,
	login: string,
	password: string
): Promise<LoginResult> => {
	const req = ApiRequestBuilder.post(`auth/login`)
		.withJsonBody({ login: login, password: password })
		.intoRequest();
	const resp = await client.exec<LoginResp>(req);
	return resp as LoginResult;
};

type LogoutSuccess = { status: 'success'; status_code: number; data: null };
type LogoutFail = { status: 'fail'; status_code: number; data: string };
type LogoutError = { status: 'error'; status_code: number; message: string };

export type LogoutResult = LogoutSuccess | LogoutFail | LogoutError;

export const logout = async (client: ApiClient): Promise<LogoutResult> => {
	const req = ApiRequestBuilder.post(`auth/logout`).intoRequest();
	const resp = await client.exec<LoginResp>(req);
	client.removeLoginToken();
	return resp as LogoutResult;
};
