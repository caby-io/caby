<script lang="ts">
	import type { ShareAuthOptions } from '$lib/api/api_shares';
	import IconLucideShare2 from '~icons/lucide/share-2';
	import IconLucideLock from '~icons/lucide/lock';
	import IconLucideGlobe from '~icons/lucide/globe';

	let { root_name, auth }: { root_name: string; auth?: ShareAuthOptions } = $props();
</script>

<div class="share-info fx fx--col">
	<div class="badge fx fx--ac">
		<IconLucideShare2 />
		<span>Shared folder</span>
	</div>
	<h2 class="name" title={root_name}>{root_name}</h2>
	{#if auth}
		<div class="access fx fx--ac">
			{#if auth.password}
				<IconLucideLock />
				<span>Password protected</span>
			{:else if auth.open}
				<IconLucideGlobe />
				<span>Public link</span>
			{/if}
		</div>
	{/if}
</div>

<style lang="scss">
	.share-info {
		gap: 0.5rem;
		padding: 1rem;
		margin: 0.75rem;
		border-radius: 8px;
		color: var(--clr-text-0);
		background:
			linear-gradient(var(--clr-background-1), var(--clr-background-1)) padding-box,
			linear-gradient(320deg, var(--clr-primary), var(--clr-accent)) border-box;
		border: 2px solid transparent;

		.badge {
			gap: 0.4rem;
			font-size: 0.7rem;
			font-weight: 600;
			letter-spacing: 0.08em;
			text-transform: uppercase;
			color: var(--clr-accent);
		}

		.name {
			margin: 0;
			font-size: 1.1rem;
			overflow-wrap: anywhere;
		}

		.access {
			gap: 0.4rem;
			font-size: 0.8rem;
			color: var(--clr-text-2);
		}
	}
</style>
