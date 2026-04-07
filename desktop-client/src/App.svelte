<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import SetupWizard from "./lib/SetupWizard.svelte";

  let pbxUrl = $state("");
  let needsSetup = $state(true);
  let loading = $state(true);

  onMount(async () => {
    try {
      const config: any = await invoke("get_config");
      if (config && config.pbx_url) {
        pbxUrl = config.pbx_url;
        needsSetup = false;
      }
    } catch (e) {
      // Keine Config → Setup anzeigen
    }
    loading = false;
  });

  function handleSetupComplete(event: CustomEvent<{ pbxUrl: string }>) {
    pbxUrl = event.detail.pbxUrl;
    needsSetup = false;
  }
</script>

{#if loading}
  <div class="loading">
    <p>Lade Konfiguration...</p>
  </div>
{:else if needsSetup}
  <SetupWizard on:complete={handleSetupComplete} />
{:else}
  <div class="loading">
    <p>4dy Client wird geladen...</p>
  </div>
{/if}

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background: #1a1a2e;
    color: #e0e0e0;
    overflow: hidden;
  }

  :global(#app) {
    height: 100vh;
  }

  :global(*) {
    box-sizing: border-box;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    color: #888;
  }
</style>
