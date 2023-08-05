<script>
    let videoElement;
    let videoStream;
    async function getCameraFeed() {
        try {
            videoStream = await navigator.mediaDevices.getUserMedia({ video: true });
            videoElement.srcObject = videoStream;
        } catch (error) {
            console.log("Error in getting camera feed: ", error);
        }
    }

    // Stop the camera feed
    function stopCameraFeed() {
        if (videoStream) {
            videoStream.getTracks().forEach((track) => track.stop());
            videoElement.srcObject = null;
        }
    }
</script>

<div>
    <video bind:this={videoElement} autoplay playsinline></video>
    <button on:click="{getCameraFeed}">Start camera</button>
    <button on:click="{stopCameraFeed}">Stop camera</button>
</div>
