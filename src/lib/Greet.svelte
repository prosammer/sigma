<script>
    import { onMount } from "svelte";

    let unlisten;

    onMount(async () => {
        const { invoke } = await import('@tauri-apps/api/tauri');
        const { listen } = await import('@tauri-apps/api/event');

        invoke('start_3pm_event_loop');

        unlisten = await listen('not-3pm-event', (event) => {
            console.log("It aint 3pm in this bitch!");
        });
    });
</script>

<div>
    <p>Greet!</p>
</div>
