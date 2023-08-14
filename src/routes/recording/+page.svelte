<script lang="ts">
    import { onMount } from "svelte";
    import { Button } from "$components/ui/button";
    import RecordRTC, { Options } from "recordrtc";
    import { appWindow } from "@tauri-apps/api/window";
    import { Store } from "tauri-plugin-store-api";

    let videoContainer: HTMLElement;
    let videoElement: HTMLVideoElement;
    let stream: MediaStream;
    let recorder: RecordRTC;

    let isRecording = false;

    onMount(async () => {
        videoElement.addEventListener('canplay', () => {
            videoContainer.style.opacity = '1';
        });
        await startCamera();
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
        const suggestedFileName = date.slice(0, 10) + "_" + date.slice(11, 13) + "_" + date.slice(14, 16) + ".webm";

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
        const { Store } = await import("tauri-plugin-store-api");

        const store = new Store(".settings.dat");

        return await store.get("videoSaveDirectory");
    }
</script>

<!-- svelte-ignore a11y-media-has-caption -->
<div id="videoContainer" bind:this={videoContainer} class="opacity-0 transition-opacity duration-500 flex flex-col items-center justify-center w-48 m-auto relative">
    <video id="videoElement" data-tauri-drag-region bind:this={videoElement} autoplay playsinline class="rounded-2xl object-cover object-center"></video>
    <button class="absolute top-0 left-0 mt-2 ml-2 bg-transparent p-1" on:click={async () => {await appWindow.close()}}>
        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-x-circle"><circle cx="12" cy="12" r="10"/><path d="m15 9-6 6"/><path d="m9 9 6 6"/></svg>
    </button>
    <div class="absolute bottom-0 flex justify-center w-full pb-4">
        {#if isRecording}
            <Button class="w-12 text-sm" on:click={stopRecording}>
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#fcffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-stop-circle"><circle cx="12" cy="12" r="10"/><rect width="6" height="6" x="9" y="9"/></svg>
            </Button>
        {:else}
            <Button class="w-12 text-sm" on:click={startRecording}>
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-video"><path d="m22 8-6 4 6 4V8Z"/><rect width="14" height="12" x="2" y="6" rx="2" ry="2"/></svg>
            </Button>
        {/if}
    </div>
</div>