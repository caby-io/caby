import { PUBLIC_API_BASE } from '$env/static/public';
import { ApiClient } from '$lib/api/client';

export const client = $state(new ApiClient(PUBLIC_API_BASE));
