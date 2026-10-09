<script lang="ts">
	import { page } from '$app/state';
	import { shares } from '$lib/stores/shares.svelte';
	import SidebarSection from './SidebarSection.svelte';
	import IconLucideShare2 from '~icons/lucide/share-2';

	const SHARE_SPEC_EXT = '.share.caby';

	const space_shares = $derived(shares.space === page.params.space ? shares.list : []);

	const baseName = (path: string) => path.split('/').filter(Boolean).pop() ?? '';

	const shareName = (spec_path: string) => {
		const name = baseName(spec_path);
		return name.endsWith(SHARE_SPEC_EXT) ? name.slice(0, -SHARE_SPEC_EXT.length) : name;
	};
</script>

{#if space_shares.length > 0}
	<SidebarSection title="Shares">
		{#each space_shares as share (share.id)}
			<a
				class="share fx fx--ac"
				class:expired={share.expired}
				href="/shares/{share.id}"
				title={share.expired ? `${share.spec_path} (expired)` : share.spec_path}
			>
				<IconLucideShare2 />
				<span class="name fx-grow ellipsis">{shareName(share.spec_path)}</span>
				<span class="target ellipsis">{baseName(share.root_entry)}/</span>
			</a>
		{/each}
	</SidebarSection>
{/if}

<style lang="scss">
	.share {
		gap: 0.5rem;
		padding: 0.4rem 0.5rem;
		border-radius: 5px;
		color: var(--clr-text-1);
		text-decoration: none;

		&:hover {
			background: var(--clr-background-2);
		}

		&.expired {
			opacity: 0.5;
		}

		.name {
			min-width: 0;
		}

		.target {
			max-width: 45%;
			font-size: 0.8rem;
			color: var(--clr-text-2);
		}
	}
</style>
