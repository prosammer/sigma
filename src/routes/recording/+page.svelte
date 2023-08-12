<script lang="ts">
    import { onMount } from "svelte";
    import { Button } from "$components/ui/button";

    let videoContainer;
    let videoElement;
    let stream;
    let recorder;

    let isRecording = false;

    onMount(async () => {
        videoElement.addEventListener('canplay', () => {
            videoContainer.style.opacity = '1';
        });
        await startCamera();
    });

    async function startCamera() {
        try {
            stream = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
            videoElement.srcObject = stream;
            videoElement.play();
        } catch (error) {
            console.error("Error accessing the camera:", error);
        }
    }

    async function startRecording() {
        isRecording = true;
        const { default: RecordRTC } = await import('recordrtc');

        recorder = RecordRTC(stream, {
            type: 'video'
        });
        recorder.startRecording();
    }

    async function stopRecording() {
        isRecording = false;
        recorder.stopRecording(async function () {
            let blob = recorder.getBlob();
            await saveVideo(blob);
        });
    }

    async function saveVideo(videoBlob) {

        const { dialog, fs } = await import('@tauri-apps/api');
        const suggestedFileName = "recording.webm";

        // TODO: Make this default to Downloads folder
        const filePath = await dialog.save({ defaultPath: suggestedFileName });

        const data = await videoBlob.arrayBuffer();
        await fs.writeBinaryFile(filePath, new Uint8Array(data));
    }
</script>

<!-- svelte-ignore a11y-media-has-caption -->
<div id="videoContainer"  bind:this={videoContainer} class="opacity-0 transition-opacity duration-500 flex flex-col items-center justify-center w-full overflow-hidden">
    <video id="videoElement" data-tauri-drag-region bind:this={videoElement} autoplay playsinline class="rounded-full object-cover object-center w-3/5 h-3/5"></video>
    {#if isRecording}
    <Button class="w-1/8 text-sm" on:click={stopRecording}>
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#fcffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-stop-circle"><circle cx="12" cy="12" r="10"/><rect width="6" height="6" x="9" y="9"/></svg>
    </Button>
    {:else}
    <Button class="w-1/8 text-sm" on:click={startRecording}>
        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-video"><path d="m22 8-6 4 6 4V8Z"/><rect width="14" height="12" x="2" y="6" rx="2" ry="2"/></svg>
    </Button>
    {/if}
</div>
