import { ApiClient } from '$lib/api/client';

export const client = $state(new ApiClient('http://localhost:8080/v0'));
