export enum Method {
	GET = 'get',
	PATCH = 'patch',
	POST = 'post',
	PUT = 'put',
	DELETE = 'delete'
}

export type ApiRequest = {
	path: string;
	method: Method;
	headers: HeadersInit;
	body?: any;
};

export class ApiRequestBuilder {
	private path?: string;
	private method?: Method;
	private headers?: HeadersInit;
	private body?: any;

	constructor(method: Method, path: string) {
		this.path = path;
		this.method = Method.GET;
		this.headers = {
			Accept: 'application/json',
			'Content-Type': 'application/json'
		};
	}

	static get(path: string) {
		return new ApiRequestBuilder(Method.GET, path);
	}

	static post(path: string) {
		return new ApiRequestBuilder(Method.POST, path);
	}

	static put(path: string) {
		return new ApiRequestBuilder(Method.PUT, path);
	}

	static delete(path: string) {
		return new ApiRequestBuilder(Method.DELETE, path);
	}

	public setPath = (path: string) => {
		this.path = path;
	};

	public setMethod = (method: Method) => {
		this.method = method;
	};

	public setHeaders = (path: string) => {
		this.path = path;
	};

	public addHeaders = (headers: HeadersInit) => {
		let joinedHeaders = new Headers(this.headers);
		let incomingHeaders = new Headers(headers);

		for (let [key, value] of incomingHeaders.entries()) {
			joinedHeaders.set(key, value);
		}

		this.headers = joinedHeaders;
	};

	public setBody = (body: any) => {
		this.body = body;
	};

	public intoRequest = (): ApiRequest => {
		return {
			path: this.path!,
			method: this.method!,
			headers: this.headers!,
			body: this.body || null
		};
	};
}

export enum ApiStatus {
	StatusSuccess = 'success',
	StatusFail = 'fail',
	StatusError = 'error'
}

export type ApiResponse<T> = {
	status: ApiStatus;
	message?: string;
	data?: T;
};

export type Token = {
	token: string;
	issued_at: Date;
	expires_at: Date;
};

export type Auth = {
	active_token?: Token;
	elevated_token?: Token;
};

// const UNHANDLED_REQUEST_ERROR: ApiResponse<null> = {
// 	status: ApiStatus.StatusError,
// 	message: 'unhandled request error'
// };

// todo: rotate session
// todo: build endpoint
export class ApiClient {
	public api_base: string;
	public auth: Auth = {};

	constructor(api_base: string) {
		this.api_base = api_base;
	}

	public exec = async <T>(req: ApiRequest): Promise<ApiResponse<T>> => {
		const body = req.body ? JSON.stringify(req.body) : null;
		try {
			const response = await fetch(`${this.api_base}/${req.path}`, {
				method: req.method,
				headers: req.headers,
				body
			});
			return await response.json();
		} catch (err) {
			console.error(`unhandled api request error: ${err}`);
			return { status: ApiStatus.StatusError, message: `unhandled request error: ${err}` };
		}
	};
}
