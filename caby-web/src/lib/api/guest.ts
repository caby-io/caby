import { createGuest } from './api_auth';
import type { ApiClient } from './client';

const GUEST_TOKEN_COOKIE = 'guest_token';

type GuestToken = {
	value: string;
	expires_at: string;
};

export const readGuestToken = async (): Promise<string | undefined> => {
	const cookie = await cookieStore.get(GUEST_TOKEN_COOKIE);
	if (!cookie?.value) {
		return undefined;
	}

	let stored: GuestToken;
	try {
		stored = JSON.parse(decodeURIComponent(cookie.value));
	} catch {
		return undefined;
	}

	// don't rely on backend since token is stateless
	if (new Date() >= new Date(stored.expires_at)) {
		await cookieStore.delete(GUEST_TOKEN_COOKIE);
		return undefined;
	}

	return stored.value;
};

export const writeGuestToken = async (value: string, expires_at: string): Promise<void> => {
	const stored: GuestToken = { value, expires_at };
	await cookieStore.set({
		name: GUEST_TOKEN_COOKIE,
		sameSite: 'strict',
		expires: new Date(expires_at).getTime(),
		value: encodeURIComponent(JSON.stringify(stored))
	});
};

export const clearGuestToken = async (client: ApiClient): Promise<void> => {
	await cookieStore.delete(GUEST_TOKEN_COOKIE);
	client.removeGuestToken();
};

export const syncGuestToken = async (client: ApiClient): Promise<string | undefined> => {
	const token = await readGuestToken();
	if (token) {
		client.setGuestToken(token);
	}
	return token;
};

export const ensureGuestToken = async (client: ApiClient): Promise<string | undefined> => {
	const existing = await readGuestToken();
	if (existing) {
		client.setGuestToken(existing);
		return existing;
	}

	const resp = await createGuest(client);
	if (resp.status !== 'success' || !resp.data) {
		console.error(`could not mint guest token: ${resp.message}`);
		return undefined;
	}

	await writeGuestToken(resp.data.guest_token, resp.data.expires_at);
	client.setGuestToken(resp.data.guest_token);
	return resp.data.guest_token;
};
