<script>
  import { onMount } from 'svelte';
  import { Card } from "$components/ui/card";
  import { Input } from "$components/ui/input";
  import { Label } from "$components/ui/label";
  import { Checkbox } from "$components/ui/checkbox";
  // import { Tabs, TabsList, TabsTrigger } from "$components/ui/tabs";

  import { enable, disable } from "tauri-plugin-autostart-api";
  import { Store } from "tauri-plugin-store-api";

  const store = new Store(".settings.dat");

  let time = "";
  let startOnLogin = false;

  onMount(async () => {
    time = await store.get("time") || "";
    startOnLogin = await store.get("startOnLogin") || false;
  });

  async function handleCheckboxChange() {
    if (startOnLogin) {
      await enable();
      console.log(`registered for autostart`);
    } else {
      await disable();
      console.log(`removed from autostart`);
    }
  }

  $: { store.set("time", time); store.save(); }
  $: { store.set("startOnLogin", startOnLogin); store.save(); handleCheckboxChange();}
</script>
<div class="w-full mx-4">
  <div class="flex flex-row">
<!--    <Tabs value="account" class="w-full flex justify-center mt-5 mb-2">-->
<!--    <TabsList>-->
<!--      <TabsTrigger value="account">General</TabsTrigger>-->
<!--      <TabsTrigger value="gpt">GPT</TabsTrigger>-->
<!--      <TabsTrigger value="schedule">Schedule</TabsTrigger>-->
<!--    </TabsList>-->
<!--  </Tabs>-->
  </div>
  <div>
    <Card class="w-5/6 mx-auto p-5 shadow-lg">
      <div class="mb-4">
        <Label for="time" class="block mb-2">When do you want to schedule a video journal?</Label>
        <Input type="time" id="time" bind:value={time} class="w-full p-2 rounded border" step="3600" />
      </div>

      <div class="mb-4 flex items-center">
        <Checkbox bind:checked={startOnLogin} id="startOnLogin" />
        <Label for="startOnLogin" class="ml-2">Start on Login</Label>
      </div>
    </Card>
  </div>
</div>