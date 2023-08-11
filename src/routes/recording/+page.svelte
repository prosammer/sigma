<script lang="ts">
    import { onMount } from "svelte";
    import { Button } from "$components/ui/button";

    let videoContainer;
    let videoElement;
    let stream;
    let recorder;

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
        const { default: RecordRTC } = await import('recordrtc');

        recorder = RecordRTC(stream, {
            type: 'video'
        });
        recorder.startRecording();
    }

    async function stopRecording() {
        recorder.stopRecording(async function () {
            let blob = recorder.getBlob();
            saveVideo(blob);
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
<div id="videoContainer" bind:this={videoContainer} class="opacity-0 transition-opacity duration-500 flex flex-col items-center justify-center w-full overflow-hidden">
    <video id="videoElement" bind:this={videoElement} autoplay playsinline class="rounded-full object-cover object-center w-3/5 h-3/5"></video>
    <div class="flex gap-2.5 mt-2.5">
        <Button on:click={startRecording}>Start Recording</Button>
        <Button on:click={stopRecording}>Stop and Save</Button>
    </div>
</div>