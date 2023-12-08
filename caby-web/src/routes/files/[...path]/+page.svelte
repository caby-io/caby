<script lang="ts">
	import { page } from '$app/stores';

	type Directory = {
		name: string;
	};

	type File = {
		name: string;
	};

	let dirs: Array<Directory> = $state([]);
	let files: Array<File> = $state([]);

	const get_data = async (path: string) => {
		const response = await fetch('http://localhost:8080/v0/files/' + path);
		const payload = await response.json();

		files = payload.data.files;
		dirs = payload.data.dirs;
	};

	$effect(() => {
		get_data($page.params.path);
	});
</script>

<section class="file-list">
	{#each dirs as dir}
		<div>
            <div class="icon">
                📁
            </div>
            <a href="files/{dir.name}">{dir.name}/</a>
        </div>
	{/each}
	{#each files as file}
		<div>
            <div class="icon">
                📃
            </div>
            {file.name}
        </div>
	{/each}
</section>

<style lang="scss">
    .file-list {
        margin: 1rem;

        > div {
            display: flex;
            // border: 1px solid var(--clr-accent);
            transition: background-color .3s, color .3s;
            font-size: 1.2em; // TEMP

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
