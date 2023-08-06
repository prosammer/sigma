<script>
    import { onMount } from "svelte";

    let videoElement;
    let stream;
    let recorder;

    onMount(async () => {
        startCamera();
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

<video bind:this={videoElement} width="640" height="480" autoplay playsinline></video>
<button on:click={startRecording}>Start Recording</button>
<button on:click={stopRecording}>Stop and Save</button>