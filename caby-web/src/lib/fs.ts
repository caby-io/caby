export const join = (...paths: Array<string>): string => {
	return paths
		.filter((p) => p != '' && p != '/' && p != null)
		.map((p) => {
			while (p.charAt(0) === '/') {
				p = p.substring(1);
			}
			return p;
		})
		.join('/');
};

export const parent = (path: string): string => {
	return path.substring(0, path.lastIndexOf('/'));
};

/**
 * Format bytes as human-readable text.
 *
 * @param bytes Number of bytes.
 * @param si True to use metric (SI) units, aka powers of 1000. False to use
 *           binary (IEC), aka powers of 1024.
 * @param dp Number of decimal places to display.
 *
 * @return Formatted string.
 */
export const prettyBytes = (bytes: number, si = false, dp = 1) => {
	const thresh = si ? 1000 : 1024;

	if (Math.abs(bytes) < thresh) {
		return bytes + ' B';
	}

	const units = si
		? ['kB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB']
		: ['KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB'];
	let u = -1;
	const r = 10 ** dp;

	do {
		bytes /= thresh;
		++u;
	} while (Math.round(Math.abs(bytes) * r) / r >= thresh && u < units.length - 1);

	return bytes.toFixed(dp) + ' ' + units[u];
};

// todo: move
export function secondsToHms(d: any) {
	d = Number(d);
	console.log(d);
	var h = Math.floor(d / 3600);
	var m = Math.floor((d % 3600) / 60);
	var s = Math.floor((d % 3600) % 60);

	var hDisplay = h > 0 ? h + 'h' : '';
	var mDisplay = m > 0 ? m + 'm' : '';
	var sDisplay = s > 0 ? s + 's' : '';
	return hDisplay + mDisplay + sDisplay;
}
