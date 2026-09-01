<script lang="ts">
	import IconLucideShare2 from '~icons/lucide/share-2';

	let { root_name, id }: { root_name: string; id: string } = $props();
</script>

<div class="share-info fx fx--col box-shadow-0-card border-0">
	<div class="badge fx fx--ac">
		<IconLucideShare2 />
		Share
	</div>
	<h2 class="name" title={root_name}>{root_name}</h2>
	<div class="id" title={id}>{id}</div>
</div>

<style lang="scss">
	.share-info {
		gap: 0.5rem;
		padding: 1rem;
		margin: 0.75rem;
		color: var(--clr-text-0);
		background-color: var(--clr-background-2);

		.badge {
			gap: 0.4rem;
			font-size: 0.8rem;
			letter-spacing: 0.08em;
			text-transform: uppercase;
			font-weight: bold;
			color: var(--clr-primary);
		}

		.name {
			margin: 0;
			font-size: 1.1rem;
			overflow-wrap: anywhere;
		}

		.id {
			font-family: monospace;
			font-size: 0.7rem;
			color: var(--clr-text-2);
			white-space: nowrap;
			overflow: hidden;
			text-overflow: ellipsis;
		}
	}
</style>
