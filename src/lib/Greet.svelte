<script>
    import { invoke } from '@tauri-apps/api/tauri'

    let prompt = ''
    let greetMsg = ''

    async function greet() {
        greetMsg = await invoke('get_completion', { prompt: prompt })
    }
</script>

<div>
    <input id="greet-input" placeholder="Enter a name..." bind:value="{prompt}" />
    <button on:click="{greet}">Greet</button>
    <p>{greetMsg}</p>
</div>