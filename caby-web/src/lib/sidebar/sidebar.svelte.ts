import { untrack, type Component, type Snippet } from 'svelte';
import { page } from '$app/state';
import { user } from '$lib/stores/user.svelte';
import SidebarFiles from './SidebarFiles.svelte';
import SidebarShares from './SidebarShares.svelte';
import SidebarSpaces from './SidebarSpaces.svelte';

export type SidebarSlot = 'main' | 'footer';

export type SidebarSection = {
	id: string;
	slot: SidebarSlot;
	component: Component;
	visible: () => boolean;
};

export type RouteSidebarSection = {
	id: string;
	slot: SidebarSlot;
	content: Snippet;
};

const inSpace = () => page.params.space !== undefined;

export const SIDEBAR_SECTIONS: SidebarSection[] = [
	{ id: 'spaces', slot: 'main', component: SidebarSpaces, visible: inSpace },
	{ id: 'files', slot: 'main', component: SidebarFiles, visible: inSpace },
	{
		id: 'shares',
		slot: 'main',
		component: SidebarShares,
		visible: () => inSpace() && user.name !== null
	}
];

class RouteSections {
	list = $state.raw<RouteSidebarSection[]>([]);

	add(section: RouteSidebarSection) {
		this.list = [...this.list.filter((s) => s.id !== section.id), section];
	}

	remove(id: string) {
		this.list = this.list.filter((s) => s.id !== id);
	}
}

export const route_sections = new RouteSections();

export const useSidebarSection = (section: RouteSidebarSection) => {
	$effect(() => {
		untrack(() => route_sections.add(section));
		return () => route_sections.remove(section.id);
	});
};
