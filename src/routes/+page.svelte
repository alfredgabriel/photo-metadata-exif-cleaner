<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { _ } from 'svelte-i18n';
  import * as ExifReader from 'exifreader';
  
  let processing = false;
  let statusMessage = '';
  let removeGps = true;
  let removeCamera = true;
  let removeDate = true;
  
  let selectedFiles: { file: File, path: string, preview: string, meta: any[] }[] = [];
  
  let isCleaned = false;

  async function processFiles(files: File[], paths: string[]) {
    statusMessage = '';
    isCleaned = false;
    selectedFiles = [];
    
    for (let i = 0; i < files.length; i++) {
        const file = files[i];
        const path = paths[i];
        const preview = URL.createObjectURL(file);
        
        let meta = [];
        try {
            const tags = await ExifReader.load(file);
            if (tags['GPSLatitude']) meta.push({ key: $_('gps'), value: $_('detected_val') });
            if (tags['Model']) meta.push({ key: $_('camera'), value: tags['Model'].description });
            if (tags['DateTimeOriginal']) meta.push({ key: $_('date_taken'), value: tags['DateTimeOriginal'].description });
            if (tags['LensModel']) meta.push({ key: $_('lens'), value: tags['LensModel'].description });
            
            if (meta.length === 0) meta.push({ key: $_('status_clean'), value: $_('clean_val') });
        } catch (err) {
            meta = [{ key: 'Info', value: $_('no_exif') }];
        }
        
        selectedFiles = [...selectedFiles, { file, path, preview, meta }];
    }
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    const files = Array.from(e.dataTransfer?.files || []);
    if (files.length === 0) return;
    
    // Attempt to extract native paths if Tauri injects them (sometimes as 'path' property)
    const paths = files.map(f => (f as any).path || '');
    if (!paths[0]) {
        alert("Drag & Drop path extraction not supported in this mode. Please use Browse Files.");
        return;
    }
    
    await processFiles(files, paths);
  }
  
  function handleDragOver(e: DragEvent) {
    e.preventDefault();
  }
  
  async function handleBrowse() {
      const selected = await open({
          multiple: true,
          filters: [{ name: 'Images', extensions: ['jpg', 'jpeg'] }]
      });
      
      if (!selected) return;
      
      const paths = Array.isArray(selected) ? selected : [selected];
      
      // We don't have the File objects directly from dialog, so we skip frontend EXIF preview for dialog selection
      // Or we can fetch them using Tauri fs API, but to keep it simple, we'll just show the paths.
      
      statusMessage = '';
      isCleaned = false;
      selectedFiles = paths.map(path => ({
          file: null as any,
          path,
          preview: '', // Hard to preview without reading file
          meta: [{ key: 'File', value: path }]
      }));
  }

  async function cleanImages() {
     processing = true;
     statusMessage = $_('cleaning');
     
     const filePaths = selectedFiles.map(s => s.path);
     
     try {
         const result = await invoke('clean_exif', { filePaths });
         statusMessage = $_('cleaned_success');
         isCleaned = true;
     } catch (err) {
         statusMessage = $_('error') + ': ' + err;
     } finally {
         processing = false;
     }
  }
  
  function reset() {
      selectedFiles = [];
      isCleaned = false;
      statusMessage = '';
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
      <h3>{$_('options')}</h3>
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
      <p>{$_('local_processing')}</p>
    </div>
  </aside>

  <!-- Main Content -->
  <main class="main-content">
    
    {#if selectedFiles.length === 0}
    <div 
      class="dropzone {processing ? 'processing' : ''}" 
      on:drop={handleDrop} 
      on:dragover={handleDragOver}
      role="button"
      tabindex="0"
    >
      <div class="drop-content">
          <svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
            <polyline points="17 8 12 3 7 8"></polyline>
            <line x1="12" y1="3" x2="12" y2="15"></line>
          </svg>
          <p class="drop-title">{$_('drag')}</p>
          <p class="drop-subtitle">{$_('or')}</p>
          <button class="browse-btn" on:click|stopPropagation={handleBrowse}>{$_('browse')}</button>
      </div>
    </div>
    {:else}
    <div class="action-area">
        <div class="metadata-section">
           <h3>{selectedFiles.length} {$_('detected')}</h3>
           
           <div class="file-list">
             {#each selectedFiles as fileData}
               <div class="file-card">
                 {#if fileData.preview}
                    <img src={fileData.preview} class="thumb" alt="thumb" />
                 {/if}
                 <div class="meta-info">
                    <p class="path-text">{fileData.path}</p>
                    <div class="metadata-grid">
                     {#each fileData.meta as item}
                       <div class="metadata-item">
                         <span class="meta-key">{item.key}</span>
                         <span class="meta-value">{item.value}</span>
                       </div>
                     {/each}
                    </div>
                 </div>
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
            
            {#if !isCleaned}
                <button class="clean-btn" on:click={cleanImages} disabled={processing}>
                   {processing ? $_('cleaning') : $_('clean')}
                </button>
            {:else}
                <button class="browse-btn" on:click={reset}>
                   {$_('clean_another')}
                </button>
            {/if}
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
    background-color: #09090b; 
    color: #fafafa;
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
    background: #18181b; 
    border-right: 1px solid #27272a; 
    display: flex;
    flex-direction: column;
    padding: 32px 24px;
    box-shadow: 2px 0 8px rgba(0,0,0,0.5);
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
    background: rgba(59, 130, 246, 0.1);
    padding: 8px;
    border-radius: 12px;
    border: 1px solid rgba(59, 130, 246, 0.2);
  }

  h1 {
    font-size: 1.25rem;
    font-weight: 700;
    margin: 0 0 4px 0;
    color: #fafafa;
  }

  .subtitle {
    color: #a1a1aa; 
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
    color: #71717a; 
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
    background: #27272a; 
  }

  .toggle input {
    display: none;
  }

  .slider {
    width: 36px;
    height: 20px;
    background-color: #3f3f46; 
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
    background-color: #fafafa;
    transition: 0.2s;
    box-shadow: 0 1px 3px rgba(0,0,0,0.3);
  }

  input:checked + .slider {
    background-color: #3b82f6;
  }

  input:checked + .slider::before {
    transform: translateX(16px);
  }

  .label-text {
    font-size: 0.95rem;
    color: #d4d4d8; 
    font-weight: 500;
  }

  .sidebar-footer {
    text-align: center;
    color: #71717a;
    font-size: 0.8rem;
    padding-top: 20px;
    border-top: 1px solid #27272a;
  }

  /* Main Content */
  .main-content {
    flex-grow: 1;
    padding: 40px;
    display: flex;
    flex-direction: column;
    background-image: radial-gradient(#27272a 1px, transparent 1px);
    background-size: 24px 24px;
    overflow-y: auto;
  }

  .dropzone {
    flex-grow: 1;
    min-height: 300px;
    border: 2px dashed #3f3f46; 
    border-radius: 16px;
    display: flex;
    justify-content: center;
    align-items: center;
    background: rgba(24, 24, 27, 0.7); 
    backdrop-filter: blur(4px);
    transition: all 0.2s ease;
    cursor: pointer;
    overflow: hidden;
  }

  .dropzone:hover {
    border-color: #3b82f6;
    background: rgba(59, 130, 246, 0.05);
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
    color: #71717a;
    margin-bottom: 16px;
  }

  .drop-title {
    font-size: 1.25rem;
    color: #fafafa;
    margin: 0 0 8px 0;
    font-weight: 600;
  }
  
  .drop-subtitle {
    color: #a1a1aa;
    margin: 0 0 24px 0;
  }

  .browse-btn {
    background: #27272a; 
    border: 1px solid #3f3f46; 
    padding: 10px 24px;
    color: #fafafa;
    border-radius: 8px;
    font-weight: 500;
    font-size: 0.95rem;
    cursor: pointer;
    box-shadow: 0 1px 2px rgba(0,0,0,0.2);
    transition: all 0.2s;
  }

  .browse-btn:hover {
    background: #3f3f46;
    border-color: #52525b;
  }

  .action-area {
    margin-top: 32px;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .metadata-section {
    background: #18181b;
    padding: 24px;
    border-radius: 12px;
    border: 1px solid #27272a;
    box-shadow: 0 4px 6px rgba(0,0,0,0.3);
  }

  .metadata-section h3 {
    font-size: 1.1rem;
    font-weight: 600;
    color: #fafafa;
    margin: 0 0 16px 0;
  }

  .file-list {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .file-card {
    display: flex;
    gap: 16px;
    background: #09090b;
    padding: 16px;
    border-radius: 8px;
    border: 1px solid #27272a;
  }

  .thumb {
    width: 80px;
    height: 80px;
    object-fit: cover;
    border-radius: 6px;
  }

  .meta-info {
    flex-grow: 1;
  }

  .path-text {
    font-family: monospace;
    font-size: 0.8rem;
    color: #71717a;
    margin: 0 0 8px 0;
    word-break: break-all;
  }

  .metadata-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .metadata-item {
    display: flex;
    gap: 8px;
    background: #18181b;
    padding: 6px 10px;
    border-radius: 4px;
    border: 1px solid #27272a;
    font-size: 0.8rem;
  }
  
  .meta-key {
    color: #a1a1aa;
    text-transform: uppercase;
    font-weight: 600;
  }
  
  .meta-value {
    color: #fafafa;
  }

  .actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    background: #18181b;
    padding: 16px 24px;
    border-radius: 12px;
    border: 1px solid #27272a;
  }

  .clean-btn {
    background: #3b82f6;
    border: none;
    padding: 12px 32px;
    color: white;
    border-radius: 8px;
    font-weight: 600;
    font-size: 1rem;
    cursor: pointer;
    transition: background 0.2s;
  }

  .clean-btn:hover:not(:disabled) {
    background: #2563eb;
  }

  .clean-btn:disabled {
    background: #3f3f46;
    color: #a1a1aa;
    cursor: not-allowed;
  }

  .status {
    padding: 8px 16px;
    border-radius: 6px;
    font-weight: 500;
    font-size: 0.95rem;
  }
  
  .status.info {
    color: #60a5fa;
  }
  
  .status.success {
    color: #34d399;
  }
</style>
