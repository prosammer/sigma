<script>
    import { onMount, onDestroy } from 'svelte';
    import 'video.js/dist/video-js.min.css';
    import 'videojs-record/dist/css/videojs.record.css';

    let videoElement;
    let videojs = undefined;
    let player = undefined;
    let RecordRTC = undefined;
    let adapter = undefined;
    let Record = undefined; // New Line


    onMount(async () => {
        const videojsImport = import('video.js');
        const recordImport = import('videojs-record/dist/videojs.record.js'); // New Line
        const recordrtcImport = import('recordrtc');
        const adapterImport = import('webrtc-adapter');
        [videojs, Record, RecordRTC, adapter] = await Promise.all([videojsImport, recordImport, recordrtcImport, adapterImport]); // Updated Line


        // options for the player
        let options = {
            controls: true,
            bigPlayButton: false,
            width: 320,
            height: 240,
            fluid: false,
            plugins: {
                record: {
                    audio: true,
                    video: true,
                    maxLength: 10,
                    debug: true
                }
            }
        };

        // initialize video.js
        player = videojs.default(videoElement, options, function() {
            var msg = 'Using video.js ' + videojs.default.VERSION +
                ' with videojs-record ' + videojs.default.getPluginVersion('record') +
                ' and recordrtc ' + RecordRTC.version;
            videojs.default.log(msg);
        });

        // error handling
        player.on('deviceError', function() {
            console.warn('device error:', player.deviceErrorCode);
        });

        player.on('error', function(element, error) {
            console.error("ERROR!!");
            console.error(error);
        });

        // user clicked the record button and started recording
        player.on('startRecord', function() {
            console.log('started recording!');
        });

        // user completed recording and stream is available
        player.on('finishRecord', function() {
            console.log('finished recording: ', player.recordedData);
        });
    });

    onDestroy(() => {
        if (player) {
            player.dispose();
        }
    });
</script>

<video id="myVideo" playsinline class="video-js vjs-default-skin" bind:this={videoElement}></video>
