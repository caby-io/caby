export const fsEntryIntoFiles = async (
	entry: FileSystemEntry,
	prefix: string = entry.name
): Promise<File[]> => {
	if (entry.isFile) {
		const file = await new Promise<File>((res, rej) =>
			(entry as FileSystemFileEntry).file(res, rej)
		);
		Object.defineProperty(file, 'webkitRelativePath', { value: prefix });
		return [file];
	}

	if (entry.isDirectory) {
		const reader = (entry as FileSystemDirectoryEntry).createReader();

		const children: FileSystemEntry[] = [];
		while (true) {
			const batch = await new Promise<FileSystemEntry[]>((res, rej) =>
				reader.readEntries(res, rej)
			);
			if (batch.length === 0) break;
			children.push(...batch);
		}

		const nested = await Promise.all(
			children.map((c) => fsEntryIntoFiles(c, `${prefix}/${c.name}`))
		);
		return nested.flat();
	}

	return [];
};
