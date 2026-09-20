import { getMe } from '$lib/api/api_auth';
import { ApiStatus, type ApiClient } from '$lib/api/client';

export const user = $state<{ name: string | null; email: string | null }>({
	name: null,
	email: null
});

export const loadUser = async (client: ApiClient) => {
	const resp = await getMe(client);
	if (resp.status === ApiStatus.StatusSuccess && resp.data) {
		user.name = resp.data.user;
		user.email = resp.data.email;
	}
};
