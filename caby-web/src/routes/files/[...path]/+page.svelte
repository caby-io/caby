<script>
	import { page } from '$app/stores';
    
    let dirs = $state([]);
    let files = $state([]);

    const get_data = async (path) => {
        const response = await fetch("http://localhost:8080/v0/files/"+path);
        const payload = await response.json();
        
        console.log(payload.data)
        files = payload.data.files;
        dirs = payload.data.dirs;
    }

    $effect(() => {
        get_data($page.params.path)
    })
</script>
{#each dirs as dir}
    <div><a href="files/{dir.name}">{dir.name}/</a></div>
{/each}
{#each files as file}
    <div>{file.name}</div>
{/each}