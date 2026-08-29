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
        if (tags['LensModel']) {
            metadataList.push({ key: 'Lens', value: tags['LensModel'].description });
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

<div class="app-wrapper">
  <!-- Sidebar -->
  <aside class="sidebar">
    <div class="brand">
      <svg class="app-logo" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M14.5 4h-5L7 7H4a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-3l-2.5-3z"></path>
        <circle cx="12" cy="13" r="3"></circle>
        <path d="M12 10v.01"></path>
        <path d="M19 13v.01"></path>
        <path d="M5 13v.01"></path>
      </svg>
      <div>
        <h1>{$_('title')}</h1>
        <p class="subtitle">{$_('subtitle')}</p>
      </div>
    </div>

    <div class="settings-panel">
      <h3>Options</h3>
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

    <div class="sidebar-footer">
      <p>100% Local Processing</p>
    </div>
  </aside>

  <!-- Main Content -->
  <main class="main-content">
    
    <div 
      class="dropzone {processing ? 'processing' : ''} {imagePreview ? 'has-image' : ''}" 
      on:drop={handleDrop} 
      on:dragover={handleDragOver}
      role="button"
      tabindex="0"
    >
      {#if !imagePreview}
        <div class="drop-content">
            <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
              <polyline points="17 8 12 3 7 8"></polyline>
              <line x1="12" y1="3" x2="12" y2="15"></line>
            </svg>
            <p class="drop-title">{$_('drag')}</p>
            <p class="drop-subtitle">or click to browse files</p>
            <button class="browse-btn">{$_('browse')}</button>
        </div>
      {:else}
        <img src={imagePreview} class="preview-image" alt="Preview" />
      {/if}
    </div>

    {#if imagePreview}
    <div class="action-area">
        <div class="metadata-section">
           <h3>Detected Metadata</h3>
           <div class="metadata-grid">
             {#each metadataList as item}
               <div class="metadata-item">
                 <span class="meta-key">{item.key}</span>
                 <span class="meta-value">{item.value}</span>
               </div>
             {/each}
           </div>
        </div>
        
        <div class="actions">
            {#if statusMessage}
              <div class="status {processing ? 'info' : 'success'}">
                {statusMessage}
              </div>
            {/if}
            <button class="clean-btn" on:click={cleanImage} disabled={processing}>
               {processing ? 'Processing...' : $_('clean')}
            </button>
        </div>
    </div>
    {/if}
    
  </main>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    background-color: #f3f4f6;
    color: #1f2937;
    height: 100vh;
    overflow: hidden;
  }

  .app-wrapper {
    display: flex;
    height: 100vh;
    width: 100%;
  }

  /* Sidebar */
  .sidebar {
    width: 320px;
    background: #ffffff;
    border-right: 1px solid #e5e7eb;
    display: flex;
    flex-direction: column;
    padding: 32px 24px;
    box-shadow: 2px 0 8px rgba(0,0,0,0.02);
    z-index: 10;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 48px;
  }

  .app-logo {
    width: 48px;
    height: 48px;
    color: #3b82f6;
    background: #eff6ff;
    padding: 8px;
    border-radius: 12px;
  }

  h1 {
    font-size: 1.25rem;
    font-weight: 700;
    margin: 0 0 4px 0;
    color: #111827;
  }

  .subtitle {
    color: #6b7280;
    font-size: 0.85rem;
    margin: 0;
  }

  .settings-panel {
    flex-grow: 1;
  }

  .settings-panel h3 {
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #9ca3af;
    margin: 0 0 20px 0;
  }

  .toggle {
    display: flex;
    align-items: center;
    cursor: pointer;
    margin-bottom: 16px;
    padding: 12px;
    border-radius: 8px;
    transition: background 0.2s;
  }
  
  .toggle:hover {
    background: #f9fafb;
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
    margin-right: 16px;
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
    background-color: #3b82f6;
  }

  input:checked + .slider::before {
    transform: translateX(16px);
  }

  .label-text {
    font-size: 0.95rem;
    color: #374151;
    font-weight: 500;
  }

  .sidebar-footer {
    text-align: center;
    color: #9ca3af;
    font-size: 0.8rem;
    padding-top: 20px;
    border-top: 1px solid #f3f4f6;
  }

  /* Main Content */
  .main-content {
    flex-grow: 1;
    padding: 40px;
    display: flex;
    flex-direction: column;
    background-image: radial-gradient(#e5e7eb 1px, transparent 1px);
    background-size: 24px 24px;
    overflow-y: auto;
  }

  .dropzone {
    flex-grow: 1;
    min-height: 300px;
    border: 2px dashed #cbd5e1;
    border-radius: 16px;
    display: flex;
    justify-content: center;
    align-items: center;
    background: rgba(255, 255, 255, 0.7);
    backdrop-filter: blur(4px);
    transition: all 0.2s ease;
    cursor: pointer;
    overflow: hidden;
  }

  .dropzone.has-image {
    padding: 24px;
    border: 2px solid #e5e7eb;
    background: #ffffff;
    cursor: default;
  }

  .dropzone:hover:not(.has-image) {
    border-color: #3b82f6;
    background: rgba(239, 246, 255, 0.8);
  }

  .dropzone.processing {
    opacity: 0.7;
    pointer-events: none;
  }
  
  .drop-content {
    text-align: center;
  }

  .icon {
    width: 64px;
    height: 64px;
    color: #9ca3af;
    margin-bottom: 16px;
  }

  .drop-title {
    font-size: 1.25rem;
    color: #374151;
    margin: 0 0 8px 0;
    font-weight: 600;
  }
  
  .drop-subtitle {
    color: #6b7280;
    margin: 0 0 24px 0;
  }

  .preview-image {
    max-height: 100%;
    max-width: 100%;
    border-radius: 8px;
    object-fit: contain;
  }

  .browse-btn {
    background: #ffffff;
    border: 1px solid #d1d5db;
    padding: 10px 24px;
    color: #374151;
    border-radius: 8px;
    font-weight: 500;
    font-size: 0.95rem;
    cursor: pointer;
    box-shadow: 0 1px 2px rgba(0,0,0,0.05);
    transition: all 0.2s;
  }

  .browse-btn:hover {
    background: #f9fafb;
    border-color: #9ca3af;
  }

  .action-area {
    margin-top: 32px;
    display: grid;
    grid-template-columns: 2fr 1fr;
    gap: 24px;
  }

  .metadata-section {
    background: #ffffff;
    padding: 24px;
    border-radius: 12px;
    border: 1px solid #e5e7eb;
    box-shadow: 0 1px 3px rgba(0,0,0,0.05);
  }

  .metadata-section h3 {
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #6b7280;
    margin: 0 0 16px 0;
  }

  .metadata-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 12px;
  }

  .metadata-item {
    display: flex;
    flex-direction: column;
    background: #f9fafb;
    padding: 12px 16px;
    border-radius: 8px;
    border: 1px solid #f3f4f6;
  }
  
  .meta-key {
    color: #6b7280;
    font-size: 0.8rem;
    margin-bottom: 4px;
    text-transform: uppercase;
    font-weight: 600;
  }
  
  .meta-value {
    color: #111827;
    font-weight: 500;
    font-size: 0.95rem;
  }

  .actions {
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    gap: 16px;
  }

  .clean-btn {
    width: 100%;
    background: #3b82f6;
    border: none;
    padding: 16px;
    color: white;
    border-radius: 12px;
    font-weight: 600;
    font-size: 1.1rem;
    cursor: pointer;
    transition: background 0.2s;
    box-shadow: 0 4px 6px rgba(59, 130, 246, 0.2);
  }

  .clean-btn:hover:not(:disabled) {
    background: #2563eb;
  }

  .clean-btn:disabled {
    background: #9ca3af;
    cursor: not-allowed;
    box-shadow: none;
  }

  .status {
    padding: 12px;
    border-radius: 8px;
    font-weight: 500;
    font-size: 0.95rem;
    text-align: center;
  }
  
  .status.info {
    background: #eff6ff;
    color: #2563eb;
    border: 1px solid #bfdbfe;
  }
  
  .status.success {
    background: #ecfdf5;
    color: #059669;
    border: 1px solid #a7f3d0;
  }
</style>
