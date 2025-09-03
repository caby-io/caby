export type Entry<T> = {
	entry_type: string;
	name: string;
	path: string;
	created_at: string;
	pretty_created_at: string;
	modified_at: string;
	pretty_modified_at: string;
	entry_fields: T;
};

export type FileFields = {
	size: number;
	pretty_size: string;
};

export type EntryProps<T> = {
	entry: Entry<T>;
	isSelected: boolean;

	// general events
	// used to select the card when appropriate
	onSelect?: () => void;

	// drag events
	onDragStart?: () => void;
	onDragEnd?: () => void;
	onDragEnter?: () => void;
	onDragOver?: () => void;
	onDragLeave?: () => void;
	onDrop?: () => void;
};
