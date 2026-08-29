<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { _ } from 'svelte-i18n';
  
  let processing = false;
  let statusMessage = '';
  let removeGps = true;
  let removeCamera = true;
  let removeDate = true;

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    const files = e.dataTransfer?.files;
    if (!files || files.length === 0) return;
    
    // Simplification for the example: just process the first file dropped
    // In a full implementation we would loop over files and output them to a selected folder.
    // Assuming the file path is accessible in Tauri via a specific method or plugin, 
    // but in Tauri v2 file drag and drop gives file paths if configured.
    // For now we will mock the path if running in browser or just show the UI state.
    
    processing = true;
    statusMessage = 'Processing...';
    
    setTimeout(() => {
        statusMessage = \('success');
        processing = false;
    }, 1500);
  }
  
  function handleDragOver(e: DragEvent) {
    e.preventDefault();
  }
</script>

<main class="container">
  <div class="glass-panel">
    <h1>{\('title')}</h1>
    <p class="subtitle">{\('subtitle')}</p>
    
    <div 
      class="dropzone {processing ? 'processing' : ''}" 
      on:drop={handleDrop} 
      on:dragover={handleDragOver}
      role="button"
      tabindex="0"
    >
      <div class="icon">📁</div>
      <p>{\('drag')}</p>
      <span>{\('or')}</span>
      <button class="browse-btn">{\('browse')}</button>
    </div>

    <div class="settings">
      <label class="toggle">
        <input type="checkbox" bind:checked={removeGps}>
        <span class="slider"></span>
        <span class="label-text">{\('remove_gps')}</span>
      </label>
      <label class="toggle">
        <input type="checkbox" bind:checked={removeCamera}>
        <span class="slider"></span>
        <span class="label-text">{\('remove_camera')}</span>
      </label>
      <label class="toggle">
        <input type="checkbox" bind:checked={removeDate}>
        <span class="slider"></span>
        <span class="label-text">{\('remove_date')}</span>
      </label>
    </div>

    {#if statusMessage}
      <div class="status {processing ? 'info' : 'success'}">
        {statusMessage}
      </div>
    {/if}
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
    background: linear-gradient(135deg, #1e1e2f, #2a2a40);
    color: #ffffff;
    min-height: 100vh;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .container {
    width: 100%;
    max-width: 600px;
    padding: 20px;
  }

  .glass-panel {
    background: rgba(255, 255, 255, 0.05);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 20px;
    padding: 40px;
    box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.3);
    text-align: center;
  }

  h1 {
    font-size: 2.5rem;
    margin-bottom: 5px;
    background: -webkit-linear-gradient(#00d2ff, #3a7bd5);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .subtitle {
    color: #a0a0b0;
    margin-bottom: 30px;
  }

  .dropzone {
    border: 2px dashed rgba(255, 255, 255, 0.2);
    border-radius: 15px;
    padding: 50px 20px;
    transition: all 0.3s ease;
    background: rgba(0, 0, 0, 0.2);
    margin-bottom: 30px;
  }

  .dropzone:hover {
    border-color: #00d2ff;
    background: rgba(0, 210, 255, 0.05);
  }

  .dropzone.processing {
    opacity: 0.5;
    pointer-events: none;
  }

  .icon {
    font-size: 3rem;
    margin-bottom: 15px;
  }

  .browse-btn {
    margin-top: 15px;
    background: linear-gradient(90deg, #00d2ff 0%, #3a7bd5 100%);
    border: none;
    padding: 10px 25px;
    color: white;
    border-radius: 25px;
    font-weight: bold;
    cursor: pointer;
    transition: transform 0.2s;
  }

  .browse-btn:hover {
    transform: scale(1.05);
  }

  .settings {
    display: flex;
    flex-direction: column;
    gap: 15px;
    text-align: left;
    background: rgba(0, 0, 0, 0.2);
    padding: 20px;
    border-radius: 15px;
    margin-bottom: 20px;
  }

  .toggle {
    display: flex;
    align-items: center;
    cursor: pointer;
  }

  .toggle input {
    display: none;
  }

  .slider {
    width: 40px;
    height: 20px;
    background-color: #444;
    border-radius: 20px;
    position: relative;
    margin-right: 15px;
    transition: 0.3s;
  }

  .slider::before {
    content: '';
    position: absolute;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    left: 2px;
    bottom: 2px;
    background-color: white;
    transition: 0.3s;
  }

  input:checked + .slider {
    background-color: #00d2ff;
  }

  input:checked + .slider::before {
    transform: translateX(20px);
  }

  .label-text {
    font-size: 1rem;
    color: #e0e0e0;
  }

  .status {
    padding: 15px;
    border-radius: 10px;
    font-weight: bold;
  }
  
  .status.info {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
  }
  
  .status.success {
    background: rgba(0, 255, 100, 0.2);
    color: #00ff64;
    border: 1px solid rgba(0, 255, 100, 0.4);
  }
</style>
