import type { Component } from 'svelte';

export type OverviewEntry = {
	entry_type: string;
	icon?: Component;
	name: string;
	path: string; // relative path of the file within the space
	children: Array<OverviewEntry>;
	is_expanded?: boolean;
	is_selected?: boolean;
};
