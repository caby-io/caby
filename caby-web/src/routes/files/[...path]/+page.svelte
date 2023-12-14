<script lang="ts">
	import { page } from '$app/stores';

	type Directory = {
		name: string;
	};

	type File = {
		name: string;
	};

	type Entry = {
		dirs: Array<Directory>,
		files: Array<File>
	}

	let entries: Entry = $state({dirs: [], files: []});

	const get_data = async (path: string) => {
		const response = await fetch('http://localhost:8080/v0/files/' + path);
		const payload = await response.json();

		entries = payload.data;
	};

	$effect(() => {
		get_data($page.params.path);
	});
</script>

<main>
	<section class="sidebar">Test</section>

	<main class="file-list">
		<div class="top-bar">
			<div class="location">
				
			</div>
		</div>
		<div class="entries">
			{#each entries.dirs as dir}
			<div>
				<div class="icon">📁</div>
				<a href="/files/{$page.params.path}/{dir.name}">{dir.name}/</a>
			</div>
		{/each}
		{#each entries.files as file}
			<div>
				<div class="icon">📃</div>
				{file.name}
			</div>
		{/each}
		</div>
	</main>
</main>

<style lang="scss">
	main {
		display: flex;
		flex-direction: row;
	}

	.sidebar {
        width: 20rem;
	}

	.file-list {
		flex-grow: 1;
	}

	.entries {
		margin: 1rem;
        flex-grow: 1;
		> div {
			display: flex;
			// border: 1px solid var(--clr-accent);
			transition:
				background-color 0.3s,
				color 0.3s;
			font-size: 1.2em; // TEMP
			padding: .5rem;
			border-radius: 3px;

			> .icon {
				width: 1.5em;
			}

			&:hover {
				color: var(--clr-background);
				background-color: var(--clr-secondary);

				a {
					color: var(--clr-background);
				}
			}
		}
	}
</style>
