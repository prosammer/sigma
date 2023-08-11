<script>
    import { onMount } from "svelte";

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
<div bind:this={videoContainer} class="video-container">
    <video  id="videoElement" bind:this={videoElement} autoplay playsinline></video>
    <div class="controls">
        <button on:click={startRecording}>Start Recording</button>
        <button on:click={stopRecording}>Stop and Save</button>
    </div>
</div>

<style>
    .video-container {
        opacity: 0;
        transition: opacity 0.5s;
        display: flex;
        flex-direction: column;
        align-items: center;
        width: 80vw;
        height: 80vw;
        overflow: hidden;
    }

    #videoElement {
        border-radius: 50%;
        object-fit: cover;
        width: 60%;
        height: 60%;
    }

    .controls {
        display: flex;
        gap: 10px;
        margin-top: 10px;
    }

    button {
        padding: 5px 10px;
    }

</style>
