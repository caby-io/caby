import { env } from '$env/dynamic/public';
import { ApiClient } from '$lib/api/client';

export const client = $state(new ApiClient({ api_base: env.PUBLIC_API_BASE ?? 'http://localhost:8080/v0' }));
