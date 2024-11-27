<script lang="ts">
	import { join } from '$lib/helpers';

	type FileEntry = {
		entry_type: string;
		name: string;
		path: string;
		created_at: string;
		pretty_created_at: string;
		modified_at: string;
		pretty_modified_at: string;
		entry_fields: {
			size: number;
			pretty_size: string;
		};
	};

	export let file_entry: FileEntry;
</script>

<tr>
	<td data-cell="select" class="check"
		><iconify-icon icon="lucide:square-check-big"></iconify-icon></td
	>
	<td data-cell="main" class="main fx">
		<div class="icon fx fx-cc">📃</div>
		<div class="text fx-grow">
			<div class="name">{file_entry.name}</div>
			<div class="size">{file_entry.entry_fields.pretty_size}</div>
		</div>
	</td>
	<td data-cell="actions" class="actions">
		<div class="fx fx-ac">
			<!-- {#if !file.symlink}
				<a
					class="action fx fx-cc"
					href={'http://localhost:8080/v0' + join('download', filesResponse.path!, file.name)}
					download={file.name}
				>
					<iconify-icon icon="lucide:hard-drive-download"></iconify-icon>
				</a>
			{:else}
				<a class="action action--invisible fx fx-cc">
					<iconify-icon icon="lucide:hard-drive-download"></iconify-icon>
				</a>
			{/if} -->
			<div class="action fx fx-cc">
				<iconify-icon icon="lucide:info"></iconify-icon>
			</div>
			<div class="action fx fx-cc">
				<iconify-icon icon="lucide:trash-2"></iconify-icon>
			</div>
			<div class="action fx fx-cc">
				<iconify-icon icon="lucide:more-horizontal"></iconify-icon>
			</div>
		</div>
	</td>
	<td data-cell="last-modified">{file_entry.pretty_modified_at}</td>
	<td data-cell="size">{file_entry.entry_fields.pretty_size}</td>
</tr>

<style lang="scss">
	tr {
		border-radius: 3px;
	}
	td {
		padding: 0.5rem;
		text-align: left;
	}

	td.check {
		width: 2rem;
	}

	td.main {
		display: flex;

		.icon {
			font-size: 1.75em;
			width: 2em;
			position: relative;

			.indicator {
				position: absolute;
				display: inline-flex;
				bottom: 0.25rem;
				right: 0.25rem;
				background: rgba(255, 255, 255, 0.8);
				border-radius: 50%;
				padding: 0.2rem;

				&--symlink {
					iconify-icon {
						font-size: 1rem;
					}
				}

				&--broken-symlink {
					iconify-icon {
						font-size: 1rem;
						color: red;
					}
				}
			}

			a {
				text-decoration: none;
			}
		}

		.name {
			width: 60%;
		}
	}

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

			&--invisible {
				opacity: 0 !important;
				pointer-events: none;
			}
		}
	}

	tbody > tr {
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
</style>
