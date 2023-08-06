<script>
    import VideoWindow from "$lib/VideoWindow.svelte";
    // import { listen } from '@tauri-apps/api/event';
    //
    // let showVideo = false;
    //
    // // Listening to the event from the backend
    // listen('show-video-window', () => {
    //     showVideo = true;
    // });
</script>

<h1>Welcome to Sigma</h1>
<p>I am the captain now.</p>
<!--{#if showVideo}-->
    <VideoWindow />
<!--{/if}-->
