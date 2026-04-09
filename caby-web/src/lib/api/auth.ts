import type { ApiResponse } from './client';

const HTTP_STATUS_UNAUTHORIZED = 401;

export const handleReauth = (goto: any, response: ApiResponse<any>, redirect?: URL) => {
	if (response.status_code === HTTP_STATUS_UNAUTHORIZED) {
		let redirect_string = '';
		if (redirect) {
			// todo: append search params
			redirect_string = `?redirect=${encodeURIComponent(redirect.pathname + redirect.search)}`;
		}
		goto(`/login${redirect_string}`);
	}
};
