<script>
  import { onMount } from 'svelte';
  import { Card } from "$components/ui/card";
  import { Input } from "$components/ui/input";
  import { Label } from "$components/ui/label";
  import { Checkbox } from "$components/ui/checkbox";

  import { enable, disable } from "tauri-plugin-autostart-api";
  import { Store } from "tauri-plugin-store-api";

  const store = new Store(".settings.dat");

  let time = "";
  let startOnLogin = false;

  onMount(async () => {
    const storedTime = await store.get("time");
    const storedStartOnLogin = await store.get("startOnLogin");

    time = storedTime || "15:00";
    console.log(storedTime ? `Time is already set: ${time}` : "Setting time to 15:00");

    startOnLogin = storedStartOnLogin || false;

    if (!storedTime) await store.set("time", time);
    if (!storedStartOnLogin) await store.set("startOnLogin", startOnLogin);
  });

  async function handleCheckboxChange() {
    if (startOnLogin) {
      await enable();
      console.log(`registered for autostart`);
    } else {
      await disable();
      console.log(`removed from autostart`);
    }

    await store.set("startOnLogin", startOnLogin);
    await store.save();
  }

  async function handleTimeChange() {
    try {
      await store.set("time", time);
      await store.save();
      console.log("Time retrieved after save:" + await store.get("time"));
    } catch (error) {
      console.error("Failed to save time to store:", error);
    }
  }

</script>
<div class="w-full mx-4">
  <div>
    <Card class="w-5/6 mx-auto p-5 shadow-lg">
      <div class="mb-4">
        <Label for="time" class="block mb-2">When do you want to schedule a video journal?</Label>
        <input type="time" id="time" bind:value={time} on:change={handleTimeChange} class="w-full p-2 rounded border"/>
      </div>

      <div class="mb-4 flex items-center">
        <Checkbox bind:checked={startOnLogin} on:change={handleCheckboxChange} id="startOnLogin" />
        <Label for="startOnLogin" class="ml-2">Start on Login</Label>
      </div>
    </Card>
  </div>
</div>