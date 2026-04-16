import { client } from '$lib/stores/client.svelte';
import { ApiRequestBuilder, type ApiClient, type Token } from './client';

export type LoginData = {
	user: string;
	login_token: Token;
};

type LoginSuccess = { status: 'success'; status_code: number; data: LoginData };
type LoginFail = { status: 'fail'; status_code: number; data: string };
type LoginError = { status: 'error'; status_code: number; message: string };

export type LoginResponse = LoginSuccess | LoginFail | LoginError;

export const login = async (
	client: ApiClient,
	login: string,
	password: string
): Promise<LoginResponse> => {
	const req = ApiRequestBuilder.post(`auth/login`).withJsonBody({ login, password }).intoRequest();
	const resp = await client.exec<LoginData>(req);
	return resp as LoginResponse;
};

export type TokenLookupData = {
	username: string;
};

type TokenLookupSuccess = { status: 'success'; status_code: number; data: TokenLookupData };
type TokenLookupFail = { status: 'fail'; status_code: number; data: string };
type TokenLookupError = { status: 'error'; status_code: number; message: string };

export type TokenLookupResponse = TokenLookupSuccess | TokenLookupFail | TokenLookupError;

export const tokenLookup = async (
	client: ApiClient,
	activation_token: string
): Promise<TokenLookupResponse> => {
	const req = ApiRequestBuilder.post(`auth/token/lookup`)
		.withJsonBody({ activation_token })
		.intoRequest();
	const resp = await client.exec<TokenLookupResponse>(req);
	return resp as TokenLookupResponse;
};

type ActivationSuccess = { status: 'success'; status_code: number; data: string };
type ActivationFail = { status: 'fail'; status_code: number; data: string };
type ActivationError = { status: 'error'; status_code: number; message: string };

export type ActivationResponse = ActivationSuccess | ActivationFail | ActivationError;

export const tokenActivation = async (
	client: ApiClient,
	activation_token: string,
	password: string
): Promise<TokenLookupResponse> => {
	const req = ApiRequestBuilder.post(`auth/token/activate`)
		.withJsonBody({ activation_token, password })
		.intoRequest();
	const resp = await client.exec<TokenLookupResponse>(req);
	return resp as TokenLookupResponse;
};

type LogoutSuccess = { status: 'success'; status_code: number; data: null };
type LogoutFail = { status: 'fail'; status_code: number; data: string };
type LogoutError = { status: 'error'; status_code: number; message: string };

export type LogoutResponse = LogoutSuccess | LogoutFail | LogoutError;

export const logout = async (client: ApiClient): Promise<LogoutResponse> => {
	const req = ApiRequestBuilder.post(`auth/logout`).intoRequest();
	const resp = await client.exec<LoginData>(req);
	client.removeLoginToken();
	return resp as LogoutResponse;
};
