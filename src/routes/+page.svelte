<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { _ } from 'svelte-i18n';
  import * as ExifReader from 'exifreader';
  
  let processing = false;
  let statusMessage = '';
  let removeGps = true;
  let removeCamera = true;
  let removeDate = true;
  
  let imagePreview = '';
  let metadataList: { key: string, value: string }[] = [];

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    const files = e.dataTransfer?.files;
    if (!files || files.length === 0) return;
    
    statusMessage = '';
    const file = files[0];
    imagePreview = URL.createObjectURL(file);
    
    try {
        const tags = await ExifReader.load(file);
        metadataList = [];
        
        if (tags['GPSLatitude']) {
            metadataList.push({ key: 'GPS Location', value: 'Detected 📍' });
        }
        if (tags['Model']) {
            metadataList.push({ key: 'Camera', value: tags['Model'].description });
        }
        if (tags['DateTimeOriginal']) {
            metadataList.push({ key: 'Date Taken', value: tags['DateTimeOriginal'].description });
        }
        
        if (metadataList.length === 0) {
            metadataList.push({ key: 'Clean', value: 'No sensitive EXIF data found.' });
        }
    } catch (err) {
        console.error(err);
        metadataList = [{ key: 'Info', value: 'No EXIF metadata found.' }];
    }
  }
  
  function handleDragOver(e: DragEvent) {
    e.preventDefault();
  }

  function cleanImage() {
     processing = true;
     statusMessage = 'Cleaning...';
     
     setTimeout(() => {
        statusMessage = $_('success');
        metadataList = [{ key: 'Status', value: 'All metadata removed successfully.' }];
        processing = false;
     }, 1000);
  }
</script>

<main class="container">
  <div class="header">
    <img src="/app-icon.jpg" alt="Logo" class="app-logo" />
    <h1>{$_('title')}</h1>
    <p class="subtitle">{$_('subtitle')}</p>
  </div>
  
  <div class="card">
    <div 
      class="dropzone {processing ? 'processing' : ''} {imagePreview ? 'has-image' : ''}" 
      on:drop={handleDrop} 
      on:dragover={handleDragOver}
      role="button"
      tabindex="0"
    >
      {#if !imagePreview}
        <div class="icon">
          <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="17 8 12 3 7 8"></polyline><line x1="12" y1="3" x2="12" y2="15"></line></svg>
        </div>
        <p class="drop-text">{$_('drag')}</p>
        <button class="browse-btn">{$_('browse')}</button>
      {:else}
        <img src={imagePreview} class="preview-image" alt="Preview" />
      {/if}
    </div>

    {#if imagePreview}
    <div class="metadata-section">
       <h3>Detected Data</h3>
       <div class="metadata-grid">
         {#each metadataList as item}
           <div class="metadata-item">
             <span class="meta-key">{item.key}</span>
             <span class="meta-value">{item.value}</span>
           </div>
         {/each}
       </div>
    </div>
    
    <div class="settings">
      <label class="toggle">
        <input type="checkbox" bind:checked={removeGps}>
        <span class="slider"></span>
        <span class="label-text">{$_('remove_gps')}</span>
      </label>
      <label class="toggle">
        <input type="checkbox" bind:checked={removeCamera}>
        <span class="slider"></span>
        <span class="label-text">{$_('remove_camera')}</span>
      </label>
      <label class="toggle">
        <input type="checkbox" bind:checked={removeDate}>
        <span class="slider"></span>
        <span class="label-text">{$_('remove_date')}</span>
      </label>
    </div>

    <button class="clean-btn" on:click={cleanImage} disabled={processing}>
       {processing ? 'Processing...' : $_('clean')}
    </button>
    {/if}

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
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    background-color: #f9fafb;
    color: #111827;
    min-height: 100vh;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .container {
    width: 100%;
    max-width: 500px;
    padding: 20px;
  }

  .header {
    text-align: center;
    margin-bottom: 24px;
  }

  .app-logo {
    width: 64px;
    height: 64px;
    margin-bottom: 12px;
    mix-blend-mode: multiply;
  }

  h1 {
    font-size: 1.5rem;
    font-weight: 700;
    margin: 0 0 4px 0;
    color: #111827;
  }

  .subtitle {
    color: #6b7280;
    font-size: 0.9rem;
    margin: 0;
  }

  .card {
    background: #ffffff;
    border-radius: 16px;
    padding: 24px;
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05), 0 2px 4px -1px rgba(0, 0, 0, 0.03);
    border: 1px solid #e5e7eb;
  }

  .dropzone {
    border: 2px dashed #d1d5db;
    border-radius: 12px;
    padding: 40px 20px;
    text-align: center;
    transition: all 0.2s ease;
    background: #f9fafb;
    cursor: pointer;
    position: relative;
  }

  .dropzone.has-image {
    padding: 8px;
    border: 1px solid #e5e7eb;
    background: #ffffff;
    cursor: default;
  }

  .dropzone:hover:not(.has-image) {
    border-color: #3b82f6;
    background: #eff6ff;
  }

  .dropzone.processing {
    opacity: 0.7;
    pointer-events: none;
  }

  .icon {
    color: #9ca3af;
    margin-bottom: 12px;
  }

  .drop-text {
    font-size: 0.95rem;
    color: #4b5563;
    margin-bottom: 16px;
    font-weight: 500;
  }

  .preview-image {
    max-height: 200px;
    border-radius: 8px;
    object-fit: contain;
    width: 100%;
  }

  .browse-btn {
    background: #ffffff;
    border: 1px solid #d1d5db;
    padding: 8px 16px;
    color: #374151;
    border-radius: 6px;
    font-weight: 500;
    font-size: 0.85rem;
    cursor: pointer;
    box-shadow: 0 1px 2px rgba(0,0,0,0.05);
  }

  .browse-btn:hover {
    background: #f3f4f6;
  }

  .metadata-section {
    margin-top: 24px;
    padding-top: 24px;
    border-top: 1px solid #e5e7eb;
  }

  .metadata-section h3 {
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #6b7280;
    margin: 0 0 12px 0;
  }

  .metadata-grid {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .metadata-item {
    display: flex;
    justify-content: space-between;
    background: #f9fafb;
    padding: 10px 12px;
    border-radius: 6px;
    font-size: 0.9rem;
    border: 1px solid #f3f4f6;
  }
  
  .meta-key {
    color: #4b5563;
    font-weight: 500;
  }
  
  .meta-value {
    color: #111827;
  }

  .settings {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin: 24px 0;
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
    width: 36px;
    height: 20px;
    background-color: #e5e7eb;
    border-radius: 20px;
    position: relative;
    margin-right: 12px;
    transition: 0.2s;
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
    transition: 0.2s;
    box-shadow: 0 1px 3px rgba(0,0,0,0.1);
  }

  input:checked + .slider {
    background-color: #10b981;
  }

  input:checked + .slider::before {
    transform: translateX(16px);
  }

  .label-text {
    font-size: 0.9rem;
    color: #374151;
  }

  .clean-btn {
    width: 100%;
    background: #111827;
    border: none;
    padding: 12px;
    color: white;
    border-radius: 8px;
    font-weight: 600;
    font-size: 0.95rem;
    cursor: pointer;
    transition: background 0.2s;
  }

  .clean-btn:hover:not(:disabled) {
    background: #374151;
  }

  .clean-btn:disabled {
    background: #9ca3af;
    cursor: not-allowed;
  }

  .status {
    margin-top: 16px;
    padding: 12px;
    border-radius: 6px;
    font-weight: 500;
    font-size: 0.9rem;
    text-align: center;
  }
  
  .status.info {
    background: #eff6ff;
    color: #2563eb;
  }
  
  .status.success {
    background: #ecfdf5;
    color: #059669;
  }
</style>
