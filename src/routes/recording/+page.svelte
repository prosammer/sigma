<script lang="ts">
    import { onMount } from "svelte";
    import RecordRTC, { Options } from "recordrtc";
    import { appWindow } from "@tauri-apps/api/window";
    import { Store } from "tauri-plugin-store-api";

    let videoContainer: HTMLElement;
    let videoElement: HTMLVideoElement;
    let stream: MediaStream;
    let recorder: RecordRTC;
    let isSystemThemeDark = false;

    let isRecording = false;

    onMount(async () => {
        videoElement.addEventListener('canplay', () => {
            videoContainer.style.opacity = '1';
        });
        await startCamera();

        isSystemThemeDark = await appWindow.theme() === "dark";

    });

    async function startCamera(): Promise<void> {
        let constraints = {
            audio: true,
            video: {
                width: 480,
                height: 480
            }
        };

        try {
            stream = await navigator.mediaDevices.getUserMedia(constraints);
            videoElement.srcObject = stream;
            await videoElement.play();
        } catch (error) {
            console.error("Error accessing the camera:", error);
        }
    }

    async function startRecording(): Promise<void> {
        isRecording = true;

        const options: Options = {
            type: 'video',
            mimeType: 'video/webm'
        };
        recorder = new RecordRTC(stream, options);
        recorder.startRecording();
    }

    async function stopRecording(): Promise<void> {
        isRecording = false;
        recorder.stopRecording(async function () {
            let blob: Blob = recorder.getBlob();
            await saveVideo(blob);
        });
    }

    async function saveVideo(videoBlob: Blob): Promise<void> {
        const { dialog, fs } = await import('@tauri-apps/api');
        const suggestedDir = await getvideoSaveDirectory() || "~/Documents/Video Journals/";

        const date = new Date().toISOString();
        const suggestedFileName = date.slice(0, 10) + "_" + date.slice(11, 13) + "_" + date.slice(14, 16) + "_" + date.slice(17, 19) + ".webm";

        console.log(suggestedDir + suggestedFileName)
        const filePath: string | null = await dialog.save({ defaultPath: suggestedDir + suggestedFileName });

        if(filePath) {
            const data: ArrayBuffer = await videoBlob.arrayBuffer();
            await fs.writeBinaryFile(filePath, new Uint8Array(data));
        } else {
            console.log("No file path selected");
        }
    }

    async function getvideoSaveDirectory(): Promise<string | null> {

        const store = new Store(".settings.dat");

        return await store.get("videoSaveDirectory");
    }
</script>

<!-- svelte-ignore a11y-media-has-caption -->
<div id="videoContainer" bind:this={videoContainer} class="opacity-0 transition-opacity duration-500 flex flex-col items-center justify-center w-48 m-auto relative">
    <video id="videoElement" data-tauri-drag-region bind:this={videoElement} autoplay playsinline class="rounded-2xl object-cover object-center"></video>
    <button class="absolute top-0 left-0 mt-2 ml-2 bg-transparent p-1" on:click={async () => {await appWindow.close()}}>
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 20 20" fill="none" stroke={isSystemThemeDark ? 'white' : 'black'} stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-x"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
    </button>
    <div class="absolute bottom-0 flex justify-center w-full h-10 items-center">
        {#if isRecording}
            <div class="w-12 flex justify-center h-full items-center" on:click={stopRecording}>
                <svg class="lucide lucide-circle red_svg" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="#CD0000" stroke="#CD0000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/></svg>
            </div>
        {:else}
            <div class="w-12 flex justify-center h-full items-center" on:click={startRecording}>
                <svg class="lucide lucide-circle" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="#00cd00" stroke="#00cd00" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/></svg>
            </div>
        {/if}

    </div>
</div>
<style>
    .red_svg {
        filter: drop-shadow(0 0 4px #CD0000);
    }
</style>