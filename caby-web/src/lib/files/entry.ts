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
};

export type FileFields = {
	size: number;
	pretty_size: string;
};

export type DirFields = {
	//todo
};

export type EntryProps<T> = {
	entry: Entry<T>;

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
};

export type DragTarget = {
	entry?: Entry;
	count: number;
};
