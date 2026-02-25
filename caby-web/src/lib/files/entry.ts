export type Entry<T = any> = {
	entry_type: string;
	name: string;
	path: string;
	created_at: string;
	pretty_created_at: string;
	modified_at: string;
	pretty_modified_at: string;
	entry_fields: T;

	// extra fields for frontend
	is_selected: boolean;
	is_targetted: boolean;
	is_processing: boolean;
};

export type FileFields = {
	size: number;
	pretty_size: string;
};

export type DirFields = {
	sub_entries: Entry[];
};

export type EntryProps<T> = {
	entry: Entry<T>;
	space?: string;

	// general events
	// used to select the card when appropriate
	onSelect?: (e: MouseEvent) => void;

	// drag events
	onDragStart?: (e: DragEvent, entry: Entry) => void;
	onDragEnd?: (e: DragEvent, entry: Entry) => void;
	onDragEnter?: (e: DragEvent, entry: Entry) => void;
	onDragOver?: (e: DragEvent, entry: Entry) => void;
	onDragLeave?: (e: DragEvent, entry: Entry) => void;
	onDrop?: (e: DragEvent, entry: Entry) => void;

	// contextmenu events
	onContextMenu?: (e: MouseEvent, entry: Entry) => void;
};

export type DragTarget = {
	entry?: Entry;
	count: number;
};

export const getDownloadURL = (base_url: String, entries: Entry[]): string => {
	if (entries.length > 1) {
		// todo
		console.error('multi-download not implemented');
		return '';
	}

	return `${base_url}/v0/files/download/${entries[0].path}`;
};
