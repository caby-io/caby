<script lang="ts">
	import { page } from '$app/state';
	import { join } from '$lib/fs';

	let { space }: { space: string } = $props();

	const dirs = $derived(page.params.path!.split('/'));

	const getPath = (index: number) => {
		return `/${join('files', space, dirs.slice(0, index + 1).join('/'))}`;
	};
</script>

<div class="breadcrumbs fx fx--ac">
	<!-- <a class="fx fx--ac">
		<iconify-icon icon="lucide:corner-left-up"></iconify-icon>
	</a> -->
	<div class="breadcrumb fx fx--ac">
		<a class="fx fx--ac" href="/files/{space}">
			<iconify-icon icon="ci:house-02"></iconify-icon>
		</a>
	</div>
	{#each dirs as dir, i}
		<div class="breadcrumb fx fx--ac">
			<a class="fx fx--ac" href={getPath(i)}>{dir}</a>
		</div>
	{/each}
</div>

<style lang="scss">
	.breadcrumbs {
		.breadcrumb > a {
			// display: block;
			padding: 0 0.25rem;
			height: 2rem;
			text-decoration: none;
			transition: opacity 0.2s;
			opacity: 1;
			border-radius: 3px;

			&:hover {
				color: inherit;
				opacity: 0.7;
			}
		}

		div + div:before {
			font-family: serif;
			font-weight: bold;
			display: block;
			content: '/';
			margin: 0.25rem;
			opacity: 0.4;
		}
	}
</style>
