<script lang="ts">
	import { page } from '$app/stores';

	type Directory = {
		name: string;
		createdAt: string;
		prettyCreatedAt: string;
		modifiedAt: string;
		prettyModifiedAt: string;
	};

	type File = {
		name: string;
		size: number;
		prettySize: string;
		createdAt: string;
		prettyCreatedAt: string;
		modifiedAt: string;
		prettyModifiedAt: string;
	};

	type FilesResponse = {
		parentPath: string | null;
		currentPath: string;
		dirs: Array<Directory>;
		files: Array<File>;
	};

	let filesResponse: FilesResponse = $state({
		parentPath: null,
		currentPath: '',
		dirs: [],
		files: []
	});

	const get_data = async (path: string) => {
		const response = await fetch('http://localhost:8080/v0/files/' + path);
		const payload = await response.json();

		filesResponse = payload.data;
	};

	const join = (...paths: Array<string>): string => {
		let joined = "";
		paths.filter(p => p != "" && p != "/" && p != null).forEach((p) => {
			joined += `/${p}`
		});
		return joined;
	};

	$effect(() => {
		get_data($page.params.path);
	});
</script>

<div class="fx full-height">
	<section class="sidebar fx--col">
		<nav class="sidebar__nav fx-grow fx fx--col">
			<a href="/files"><h4>Files</h4></a>
			<a href="/"><h4>Home?</h4></a>
		</nav>
		<div class="usage-metrics">
			<header class="fx fx-cc">
				<h1 class="fx-grow">Disk Usage (Fake)</h1>
				<span>1GB/20GB (5%)</span>
			</header>
			<div><span></span></div>
		</div>
	</section>

	<section class="file-list">
		<section class="top-bar">
			<div class="location"></div>
		</section>
		<main class="entries">
			<table>
				<thead>
					<tr>
						<th class="icon"></th>
						<th class="name">Name</th>
						<th>Size</th>
						<th>Last Modified</th>
						<th class="actions"></th>
					</tr>
				</thead>
				<tbody>
					{#if filesResponse.parentPath != null}
						<tr>
							<td class="icon">📁</td>
							<td><a href={join("files", filesResponse.parentPath)}>..</a></td>
							<td>..</td>
							<td>..</td>
							<td></td>
						</tr>
					{/if}
					{#each filesResponse.dirs as dir}
						<tr>
							<td class="icon">📁</td>
							<!-- todo: improve -->
							<td><a href={join("files", filesResponse.currentPath, dir.name)}>{dir.name}/</a></td>
							<td>..</td>
							<td>..</td>
							<td><button>..</button></td>
						</tr>
					{/each}
					{#each filesResponse.files as file}
						<tr>
							<td class="icon">📃</td>
							<td class="name ellipsis">{file.name}</td>
							<td>{file.prettySize}</td>
							<td>{file.prettyModifiedAt}</td>
							<td><button>..</button></td>
						</tr>
					{/each}
				</tbody>
			</table>
		</main>
	</section>
</div>

<style lang="scss">
	.sidebar {
		width: 20rem;
		display: flex;
	}

	.sidebar__nav {
		> a {
			margin: 1rem 1rem 0;
			padding: 0.5rem 1rem;
			border-radius: 5px;
			color: var(--clr-background);
			background: var(--clr-secondary);
		}
	}

	.usage-metrics {
		padding: 2rem 1rem;

		> header {
			> h1 {
				font-size: 0.9rem; // todo: replace with var
				font-weight: normal;
			}

			> span {
				font-size: 0.6rem;
			}
		}

		> div {
			width: 100%;
			height: 0.2rem;
			border-radius: 0.1rem;
			display: flex;
			background: var(--clr-text);

			> span {
				// temp
				width: 5%;
				height: 100%;
				background-color: var(--clr-secondary);
			}
		}
	}

	.file-list {
		flex-grow: 1;
		height: 100%;
	}

	.entries {
		// margin: 1rem;
		flex-grow: 1;
		height: 100%;
		overflow-y: scroll;

		> table {
			border-spacing: 0rem;
			font-size: 1.1em;
			width: 100%;

			tr {
				border-radius: 3px;
			}

			td,
			th {
				padding: 0.5rem;
			}

			th {
				text-align: left;
			}

			.icon {
				width: 1.5em;
			}

			.name {
				width: 60%;
			}

			tbody > tr {
				&:hover {
					color: var(--clr-background);
					background-color: var(--clr-secondary);

					a {
						color: var(--clr-background);
					}
				}
			}
		}

		// > div {
		// 	display: flex;
		// 	// border: 1px solid var(--clr-accent);
		// 	transition:
		// 		background-color 0.3s,
		// 		color 0.3s;
		// 	font-size: 1.2em; // TEMP
		// 	padding: 0.5rem;
		// 	border-radius: 3px;

		// 	> .icon {
		// 		width: 1.5em;
		// 	}

		// 	&:hover {
		// 		color: var(--clr-background);
		// 		background-color: var(--clr-secondary);

		// 		a {
		// 			color: var(--clr-background);
		// 		}
		// 	}
		// }
	}
</style>
