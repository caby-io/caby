export type OverviewEntry = {
	entry_type: string;
	name: string;
	path: string; // relative path of the file from the mount root
	children: Array<OverviewEntry>;
	is_expanded?: boolean;
};
