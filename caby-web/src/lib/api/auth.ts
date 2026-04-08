import type { ApiResponse } from './client';

const HTTP_STATUS_UNAUTHORIZED = 401;

export const handleReauth = (goto: any, response: ApiResponse<any>) => {
	if (response.status_code === HTTP_STATUS_UNAUTHORIZED) {
		goto('/login');
	}
};
