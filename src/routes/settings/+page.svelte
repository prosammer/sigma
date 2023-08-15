<script>
  import { onMount } from 'svelte';
  import { Input } from "$components/ui/input";
  import { Label } from "$components/ui/label";
  import { Checkbox } from "$components/ui/checkbox";
  import { Button } from "$components/ui/button";

  import { open } from '@tauri-apps/api/dialog';
  import { enable, disable } from "tauri-plugin-autostart-api";
  import { Store } from "tauri-plugin-store-api";

  const store = new Store(".settings.dat");

  let time = "";
  let startOnLogin = false;
  let videoSaveDirectory = "";

  onMount(async () => {
    const storedTime = await store.get("time");
    const storedStartOnLogin = await store.get("startOnLogin");
    const storedVideoSaveDirectory = await store.get("videoSaveDirectory");

    time = storedTime || "15:00";
    startOnLogin = storedStartOnLogin || false;
    videoSaveDirectory = storedVideoSaveDirectory || "~/Movies/Video Journals/";

    if (!storedTime) await store.set("time", time);
    if (!storedStartOnLogin) await store.set("startOnLogin", startOnLogin);
    if (!storedVideoSaveDirectory) await store.set("videoSaveDirectory", videoSaveDirectory);
  });

  $: if (time !== "") store.set("time", time)

  $: startOnLogin ? enable() : disable();

  $: store.set("startOnLogin", startOnLogin);

  $: if (videoSaveDirectory !== "") {
    store.set("videoSaveDirectory", videoSaveDirectory).then(() => {
      console.log("Video save directory saved:", videoSaveDirectory);
    }).catch(error => {
      console.error("Failed to save video save directory to store:", error);
    });
  }

  async function readFolderContents() {
    try {
      const selectedFolder = await open({
        directory: true,
        multiple: false,
        defaultPath: "~/Desktop/"
      });
      if (selectedFolder !== null) videoSaveDirectory = selectedFolder + "/";
    } catch (error) {
      console.error(error);
    }
  }

</script>
<div class="w-full h-full dark:bg-[#2C2831]">
  <div class="w-5/6 mx-auto p-5 shadow-lg">
    <h1 class="pb-4 dark:text-white">General</h1>
    <div class="mb-4">
      <Label for="time" class="block mb-2 px-2 dark:text-white">When do you want to schedule a video journal?</Label>
      <input type="time" id="time" bind:value={time} class="w-full p-2 rounded border dark:bg-[#2C2831] dark:text-white dark:border-dark-mode-white"/>
    </div>

    <div class="mb-4 flex items-center">
      <Checkbox bind:checked={startOnLogin} id="startOnLogin" class="dark:outline-dark-mode-white" />
      <Label for="startOnLogin" class="ml-2 dark:text-white">Start on Login</Label>
    </div>

    <h1 class="pb-4 dark:text-white">Save</h1>
    <div class="mb-4 flex items-center">
      <Label for="startOnLogin" class="px-2 dark:text-white">Location</Label>
      <Input type="text" class="dark:text-white dark:border-dark-mode-white" disabled bind:value={videoSaveDirectory}></Input>
      <Button class="dark:bg-dark-mode-button dark:text-dark-mode-button-text" on:click={readFolderContents}>Choose Folder</Button>
    </div>
    <div class="h-80">
    </div>
  </div>
</div>