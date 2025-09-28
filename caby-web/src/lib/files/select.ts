import type { Entry } from './entry';
import type { EntryType } from './upload/upload';

export type SelectedEntry = {
	// type: EntryType;
	index: number;
	entry: Entry;
};
