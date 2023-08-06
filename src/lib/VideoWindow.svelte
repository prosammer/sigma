<script>
    import { onMount } from "svelte";
    import {dialog, fs} from "@tauri-apps/api";

    let videoElement;
    let stream;
    let recorder;
    let downloadURL;


    onMount(async () => {
        const { default: RecordRTC } = await import('recordrtc');
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
            // save the file
        });
    }
</script>

<video bind:this={videoElement} width="640" height="480" autoplay playsinline></video>
<button on:click={startRecording}>Start Recording</button>
<button on:click={stopRecording}>Stop and Save</button>