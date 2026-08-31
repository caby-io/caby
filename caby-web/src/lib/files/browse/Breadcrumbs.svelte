<script lang="ts">
	import { page } from '$app/state';
	import IconCiHouse02 from '~icons/ci/house-02';

	let { space, href_base }: { space?: string; href_base?: string } = $props();

	const base = $derived(href_base ?? `/files/${space}`);
	const dirs = $derived((page.params.path ?? '').split('/').filter(Boolean));

	const getPath = (index: number) => {
		return `${base}/${dirs.slice(0, index + 1).join('/')}`;
	};
</script>

<div class="breadcrumbs fx fx--ac">
	<div class="breadcrumb fx fx--ac">
		<a class="fx fx--ac" href={base}>
			<IconCiHouse02 class="house-icon" />
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
			padding: 0 0.25rem;
			height: 2rem;
			text-decoration: none;
			transition: opacity 0.2s;
			opacity: 1;
			border-radius: 3px;

			:global(.house-icon) {
				font-size: 0.9em;
			}

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
