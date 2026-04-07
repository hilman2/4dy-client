<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { validatePbxUrl } from "./validation";

  const dispatch = createEventDispatcher();

  let pbxUrl = $state("");
  let error = $state("");
  let saving = $state(false);

  async function handleSubmit() {
    const result = validatePbxUrl(pbxUrl);
    if (!result.ok) {
      error = result.error;
      return;
    }

    saving = true;
    error = "";

    try {
      await invoke("save_initial_config", { pbxUrl: result.value });
      dispatch("complete", { pbxUrl: result.value });
    } catch (e) {
      error = `Fehler beim Speichern: ${e}`;
      saving = false;
    }
  }
</script>

<div class="wizard">
  <div class="wizard-card">
    <h1>4dy Client einrichten</h1>
    <p>Bitte gib die URL deines 3CX Web-Clients ein:</p>

    <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
      <input
        type="url"
        bind:value={pbxUrl}
        placeholder="https://meine-firma.3cx.de"
        disabled={saving}
      />

      {#if error}
        <p class="error">{error}</p>
      {/if}

      <button type="submit" disabled={saving}>
        {saving ? "Speichere..." : "Weiter"}
      </button>
    </form>
  </div>
</div>

<style>
  .wizard {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    background: #1a1a2e;
  }

  .wizard-card {
    background: #16213e;
    border: 1px solid #0f3460;
    border-radius: 12px;
    padding: 40px;
    width: 400px;
    max-width: 90vw;
  }

  h1 {
    margin: 0 0 8px;
    font-size: 24px;
    color: #e94560;
  }

  p {
    color: #aaa;
    margin: 0 0 24px;
  }

  input {
    width: 100%;
    padding: 12px;
    border: 1px solid #0f3460;
    border-radius: 6px;
    background: #1a1a2e;
    color: #e0e0e0;
    font-size: 14px;
    box-sizing: border-box;
  }

  input:focus {
    outline: none;
    border-color: #e94560;
  }

  .error {
    color: #e94560;
    font-size: 13px;
    margin: 8px 0 0;
  }

  button {
    width: 100%;
    margin-top: 16px;
    padding: 12px;
    background: #e94560;
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 15px;
    cursor: pointer;
    transition: background 0.2s;
  }

  button:hover:not(:disabled) {
    background: #c73e54;
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
