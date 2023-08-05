<script>
    import { invoke } from '@tauri-apps/api/tauri'

    let name = ''
    let greetMsg = ''

    async function submitName() {
        greetMsg = await invoke('get_completion', { name: name })
    }
</script>

<div>
    <input id="greet-input" placeholder="Enter your name..." bind:value="{name}" />
    <button on:click="{submitName}">Enter</button>
    <p>{greetMsg}</p>
</div>