<script lang="ts">
	import { page } from '$app/stores';

	import 'iconify-icon';

	type Directory = {
		name: string;
		path: string;
		created_at: string;
		pretty_created_at: string;
		modified_at: string;
		pretty_modified_at: string;
	};

	type File = {
		name: string;
		path: string;
		size: number;
		pretty_size: string;
		created_at: string;
		pretty_created_at: string;
		modified_at: string;
		pretty_modified_at: string;
	};

	type FilesResponse = {
		path: string | null;
		parent_dir: string | null;
		current_dir: string;
		dirs: Array<Directory>;
		files: Array<File>;
	};

	let filesResponse: FilesResponse = $state({
		path: null,
		parent_dir: null,
		current_dir: '',
		dirs: [],
		files: []
	});

	let loading = $state(false);

	const get_data = async (path: string) => {
		loading = true;
		const response = await fetch('http://localhost:8080/v0/files/' + path);
		const payload = await response.json();

		filesResponse = payload.data;

		// Fix URL if it's incorrect
		// if document.location.href != join("files", response.path) {
		// 	document.location.href = join("files", response.path)
		// }

		loading = false;
		// document.location.href = join("files", response.path)
	};

	const join = (...paths: Array<string>): string => {
		let joined = '';
		paths
			.filter((p) => p != '' && p != '/' && p != null)
			.forEach((p) => {
				while (p.charAt(0) === '/') {
					p = p.substring(1);
				}
				joined += `/${p}`;
			});
		return joined;
	};

	// onMount(() => {
	// 	get_data($page.params.path);
	// })

	$effect(() => {
		get_data($page.params.path);
	});
</script>

<div class="right fx fx--col fx-grow">
	<header>breadcrumbs</header>
	<section class="file-list">
		<section class="top-bar">
			<div class="location"></div>
		</section>
		<main class="entries">
			{#if loading}
				<table class="skeleton">
					<thead>
						<tr>
							<th class="icon"></th>
							<th class="name"><span /></th>
							<th class="actions"><span /></th>
							<th><span /></th>
							<th><span /></th>
							<th class="actions"><span /></th>
						</tr>
					</thead>
					<tbody>
						{#each { length: 3 } as _, i}
							<tr>
								<th class="icon"></th>
								<th class="name"><span /></th>
								<th class="actions"><span /></th>
								<th><span /></th>
								<th><span /></th>
								<th class="actions"><span /></th>
							</tr>
						{/each}
					</tbody>
				</table>
			{:else}
				<table>
					<thead>
						<tr>
							<th class="icon"></th>
							<th class="name">Name</th>
							<th class="actions"></th>
							<th>Size</th>
							<th>Last Modified</th>
							<th class="actions"></th>
						</tr>
					</thead>
					<tbody>
						{#if filesResponse.parent_dir != null}
							<tr>
								<td class="check"></td>
								<td class="main fx">
									<div class="icon fx fx-cc">
										<a href={join('files', filesResponse.parent_dir)}>📁</a>
									</div>
									<div class="text fx-grow">
										<div class="name"><a href={join('files', filesResponse.parent_dir)}>..</a></div>
										<!-- <div class="size">Unknown</div> -->
									</div>
								</td>
								<td></td>
								<td>..</td>
								<td>..</td>
								<td></td>
							</tr>
						{/if}
						{#each filesResponse.dirs as dir}
							<tr>
								<td class="check"><iconify-icon icon="lucide:square"></iconify-icon></td>
								<!-- todo: improve -->
								<td class="main fx">
									<div class="icon fx fx-cc"><a href={join('files', dir.path)}>📁</a></div>
									<div class="text fx-grow">
										<div class="name"><a href={join('files', dir.path)}>{dir.name}/</a></div>
										<div class="size">–</div>
									</div>
								</td>
								<td class="actions"></td>
								<td>..</td>
								<td>{dir.pretty_modified_at}</td>
								<td><button>..</button></td>
							</tr>
						{/each}
						{#each filesResponse.files as file}
							<tr>
								<td class="check"><iconify-icon icon="lucide:square-check-big"></iconify-icon></td>
								<td class="main fx">
									<div class="icon fx fx-cc">📃</div>
									<div class="text fx-grow">
										<div class="name">{file.name}</div>
										<div class="size">{file.pretty_size}</div>
									</div>
								</td>
								<td class="actions">
									<div class="fx fx-ac">
										<div class="action fx fx-cc">
											<iconify-icon icon="lucide:hard-drive-download"></iconify-icon>
										</div>
										<div class="action fx fx-cc">
											<iconify-icon icon="lucide:info"></iconify-icon>
										</div>
										<div class="action fx fx-cc">
											<iconify-icon icon="lucide:trash-2"></iconify-icon>
										</div>
									</div>
								</td>
								<td>{file.pretty_size}</td>
								<td>{file.pretty_modified_at}</td>
								<td><button>..</button></td>
							</tr>
						{/each}
					</tbody>
				</table>
			{/if}
		</main>
	</section>
</div>

<style lang="scss">
	.right {
		max-height: 100vh;
	}

	.file-list {
		flex-grow: 1;
		height: 0; /* Need to investigate why this works */
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

			&.skeleton {
				span {
					display: block;
					height: 0.4rem;
					min-width: 100px;
					width: 70%;
					background: lightgrey;
					border-radius: 3px;
				}
			}

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

			.check {
				width: 2rem;
			}

			.main {
				display: flex;

				.icon {
					font-size: 1.75em;
					width: 2em;

					a {
						text-decoration: none;
					}
				}

				.name {
					width: 60%;
				}
			}

			tbody > tr {
				td.actions {
					font-size: 1.5rem;

					.action {
						cursor: pointer;
						color: var(--clr-secondary);
						margin-right: 0.5rem;
						width: 2.1rem;
						height: 2.1rem;
						background: var(--clr-accent);
						opacity: 0.6;
						border-radius: 3px;
						transition: color 0.3s;
					}
					// > div {
					// 	display: inline-block;
					// 	padding: 2px;
					// 	margin-right: 0.5rem;
					// 	background: lightgrey;
					// }
				}

				&:hover {
					color: var(--clr-background);
					background-color: var(--clr-secondary);

					a {
						color: var(--clr-background);
					}

					td.actions .action {
						opacity: 0.8;
					}
				}

				td.actions .action:hover {
					opacity: 1;
					color: var(--clr-primary);
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
