export const getScheme = (): string => {
	// todo: confirm the default
	return document.documentElement.getAttribute('data-theme') || 'light';
};

export const toggleScheme = () => {
	const new_scheme =
		document.documentElement.getAttribute('data-theme') == 'dark' ? 'light' : 'dark';
	setScheme(new_scheme);
};

export const clearStorage = () => {
	localStorage.color_scheme = null;
};

export const setScheme = (scheme: string) => {
	localStorage.color_scheme = scheme;
	document.documentElement.setAttribute('data-theme', scheme);
};
