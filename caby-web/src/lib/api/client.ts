export enum Method {
	GET = 'GET',
	POST = 'POST',
	PUT = 'PUT',
	PATCH = 'PATCH',
	DELETE = 'DELETE'
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
		this.method = method;
		this.headers = {
			Accept: 'application/json'
		};
	}

	static get(path: string): ApiRequestBuilder {
		return new ApiRequestBuilder(Method.GET, path);
	}

	static post(path: string): ApiRequestBuilder {
		return new ApiRequestBuilder(Method.POST, path);
	}

	static put(path: string): ApiRequestBuilder {
		return new ApiRequestBuilder(Method.PUT, path);
	}

	static patch(path: string): ApiRequestBuilder {
		return new ApiRequestBuilder(Method.PATCH, path);
	}

	static delete(path: string): ApiRequestBuilder {
		return new ApiRequestBuilder(Method.DELETE, path);
	}

	public setPath = (path: string): ApiRequestBuilder => {
		this.path = path;
		return this;
	};

	public withMethod = (method: Method): ApiRequestBuilder => {
		this.method = method;
		return this;
	};

	public withHeaders = (path: string): ApiRequestBuilder => {
		this.path = path;
		return this;
	};

	public addHeaders = (headers: HeadersInit): ApiRequestBuilder => {
		let joinedHeaders = new Headers(this.headers);
		let incomingHeaders = new Headers(headers);

		for (let [key, value] of incomingHeaders.entries()) {
			joinedHeaders.set(key, value);
		}

		this.headers = joinedHeaders;
		return this;
	};

	public withBody = (body: any): ApiRequestBuilder => {
		this.body = body;
		return this;
	};

	public withJsonBody = (body: any): ApiRequestBuilder => {
		this.body = JSON.stringify(body);
		this.addHeaders({ 'Content-Type': 'application/json' });
		return this;
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
	value: string;
	issued_at: Date;
	expires_at: Date;
};

export type Auth = {
	login_token?: Token;
	elevated_token?: Token;
};

// const UNHANDLED_REQUEST_ERROR: ApiResponse<null> = {
// 	status: ApiStatus.StatusError,
// 	message: 'unhandled request error'
// };

// todo: rotate session
export class ApiClient {
	public api_base: string;
	public auth: Auth = {};

	constructor(api_base: string) {
		this.api_base = api_base;
	}

	public setLoginToken = (token: Token) => {
		this.auth.login_token = token;
	};

	public isAuthenticated = async (): Promise<boolean> => {
		if (!this.auth.login_token) {
			console.log('failed here');
			return false;
		}

		if (new Date() > this.auth.login_token.expires_at) {
			console.log('or here');
			return false;
		}

		return true;
	};

	public exec = async <T>(req: ApiRequest): Promise<ApiResponse<T>> => {
		try {
			const response = await fetch(`${this.api_base}/${req.path}`, {
				method: req.method,
				headers: req.headers,
				body: req.body
			});
			return await response.json();
		} catch (err) {
			console.error(`unhandled api request error: ${err}`);
			return { status: ApiStatus.StatusError, message: `unhandled request error: ${err}` };
		}
	};
}
