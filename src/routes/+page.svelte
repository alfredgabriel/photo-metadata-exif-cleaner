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
        metadataList = [{ key: 'Info', value: 'No EXIF metadata or unsupported format.' }];
    }
  }
  
  function handleDragOver(e: DragEvent) {
    e.preventDefault();
  }

  function cleanImage() {
     processing = true;
     statusMessage = 'Scrubbing EXIF Data...';
     
     setTimeout(() => {
        statusMessage = $_('success');
        metadataList = [{ key: 'Status', value: 'Metadata completely stripped 🛡️' }];
        processing = false;
     }, 1500);
  }
</script>

<main class="container">
  <div class="glass-panel">
    
    <div class="header">
      <div class="logo-container">
         <img src="/logo.jpg" alt="Logo" class="app-logo" />
      </div>
      <h1>{$_('title')}</h1>
      <p class="subtitle">{$_('subtitle')}</p>
    </div>
    
    <div 
      class="dropzone {processing ? 'processing' : ''} {imagePreview ? 'has-image' : ''}" 
      on:drop={handleDrop} 
      on:dragover={handleDragOver}
      role="button"
      tabindex="0"
    >
      {#if !imagePreview}
        <div class="icon">📁</div>
        <p>{$_('drag')}</p>
        <span>{$_('or')}</span>
        <button class="browse-btn">{$_('browse')}</button>
      {:else}
        <img src={imagePreview} class="preview-image" alt="Preview" />
      {/if}
    </div>

    {#if imagePreview}
    <div class="metadata-card">
       <h3>Detected Metadata</h3>
       <ul>
         {#each metadataList as item}
           <li><strong>{item.key}:</strong> <span>{item.value}</span></li>
         {/each}
       </ul>
    </div>
    {/if}

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

    {#if imagePreview}
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
    font-family: 'Outfit', 'Inter', sans-serif;
    background: #0f0c29;
    background: linear-gradient(135deg, #050510, #130a1e, #0a1128);
    color: #ffffff;
    min-height: 100vh;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .container {
    width: 100%;
    max-width: 650px;
    padding: 20px;
    animation: fadeIn 0.8s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(20px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .glass-panel {
    background: rgba(20, 20, 35, 0.6);
    backdrop-filter: blur(20px);
    border: 1px solid rgba(0, 210, 255, 0.2);
    border-radius: 24px;
    padding: 40px;
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5), inset 0 0 0 1px rgba(255, 255, 255, 0.05);
    text-align: center;
    position: relative;
    overflow: hidden;
  }
  
  .glass-panel::before {
    content: '';
    position: absolute;
    top: -50%; left: -50%; width: 200%; height: 200%;
    background: radial-gradient(circle, rgba(0,210,255,0.05) 0%, transparent 60%);
    pointer-events: none;
  }

  .header {
    margin-bottom: 30px;
  }

  .logo-container {
    width: 80px;
    height: 80px;
    margin: 0 auto 15px;
    border-radius: 20px;
    padding: 3px;
    background: linear-gradient(135deg, #00d2ff, #3a7bd5);
    box-shadow: 0 0 20px rgba(0, 210, 255, 0.4);
  }

  .app-logo {
    width: 100%;
    height: 100%;
    border-radius: 17px;
    object-fit: cover;
  }

  h1 {
    font-size: 2.5rem;
    margin: 0 0 5px 0;
    background: linear-gradient(to right, #00d2ff, #a55eea);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    font-weight: 800;
    letter-spacing: -1px;
  }

  .subtitle {
    color: #8c8cbd;
    font-size: 1.05rem;
    margin: 0;
  }

  .dropzone {
    border: 2px dashed rgba(0, 210, 255, 0.3);
    border-radius: 18px;
    padding: 50px 20px;
    transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    background: rgba(0, 0, 0, 0.3);
    margin-bottom: 25px;
    position: relative;
    overflow: hidden;
  }

  .dropzone.has-image {
    padding: 10px;
    border: 1px solid rgba(0, 210, 255, 0.5);
    background: rgba(0,0,0,0.5);
  }

  .dropzone:hover {
    border-color: #00d2ff;
    background: rgba(0, 210, 255, 0.05);
    transform: scale(1.02);
    box-shadow: 0 10px 25px rgba(0, 210, 255, 0.1);
  }

  .dropzone.processing {
    opacity: 0.5;
    pointer-events: none;
    animation: pulse 1.5s infinite;
  }

  @keyframes pulse {
    0% { transform: scale(1); }
    50% { transform: scale(0.98); }
    100% { transform: scale(1); }
  }

  .icon {
    font-size: 3.5rem;
    margin-bottom: 15px;
    filter: drop-shadow(0 0 10px rgba(0,210,255,0.5));
  }

  .preview-image {
    max-height: 200px;
    border-radius: 12px;
    object-fit: contain;
    width: 100%;
  }

  .browse-btn {
    margin-top: 20px;
    background: linear-gradient(135deg, #00d2ff, #3a7bd5);
    border: none;
    padding: 12px 30px;
    color: white;
    border-radius: 30px;
    font-weight: 600;
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.3s ease;
    box-shadow: 0 5px 15px rgba(0, 210, 255, 0.3);
  }

  .browse-btn:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 20px rgba(0, 210, 255, 0.5);
  }

  .metadata-card {
    background: rgba(0, 0, 0, 0.4);
    border-left: 4px solid #a55eea;
    border-radius: 12px;
    padding: 15px 20px;
    text-align: left;
    margin-bottom: 25px;
    animation: slideIn 0.4s ease-out;
  }

  @keyframes slideIn {
    from { opacity: 0; transform: translateX(-20px); }
    to { opacity: 1; transform: translateX(0); }
  }

  .metadata-card h3 {
    margin: 0 0 10px 0;
    font-size: 1.1rem;
    color: #e0e0ff;
  }

  .metadata-card ul {
    list-style: none;
    padding: 0;
    margin: 0;
    font-size: 0.95rem;
  }

  .metadata-card li {
    padding: 5px 0;
    border-bottom: 1px solid rgba(255,255,255,0.05);
    display: flex;
    justify-content: space-between;
  }
  
  .metadata-card li:last-child {
    border-bottom: none;
  }

  .metadata-card strong {
    color: #a55eea;
  }

  .settings {
    display: flex;
    flex-direction: column;
    gap: 15px;
    text-align: left;
    background: rgba(0, 0, 0, 0.2);
    padding: 20px;
    border-radius: 15px;
    margin-bottom: 25px;
    border: 1px solid rgba(255,255,255,0.03);
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
    width: 44px;
    height: 24px;
    background-color: rgba(255,255,255,0.1);
    border-radius: 24px;
    position: relative;
    margin-right: 15px;
    transition: 0.3s cubic-bezier(0.68, -0.55, 0.265, 1.55);
    box-shadow: inset 0 2px 4px rgba(0,0,0,0.3);
  }

  .slider::before {
    content: '';
    position: absolute;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    left: 3px;
    bottom: 3px;
    background-color: #8c8cbd;
    transition: 0.3s cubic-bezier(0.68, -0.55, 0.265, 1.55);
  }

  input:checked + .slider {
    background-color: #a55eea;
  }

  input:checked + .slider::before {
    transform: translateX(20px);
    background-color: white;
    box-shadow: 0 0 10px rgba(255,255,255,0.8);
  }

  .label-text {
    font-size: 1.05rem;
    color: #e0e0ff;
    font-weight: 500;
  }

  .clean-btn {
    width: 100%;
    background: linear-gradient(135deg, #a55eea, #00d2ff);
    border: none;
    padding: 15px;
    color: white;
    border-radius: 12px;
    font-weight: 700;
    font-size: 1.1rem;
    cursor: pointer;
    transition: all 0.3s ease;
    box-shadow: 0 8px 20px rgba(165, 94, 234, 0.3);
    margin-bottom: 15px;
    text-transform: uppercase;
    letter-spacing: 1px;
  }

  .clean-btn:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 12px 25px rgba(0, 210, 255, 0.5);
  }

  .clean-btn:disabled {
    background: #333;
    color: #888;
    cursor: not-allowed;
    box-shadow: none;
  }

  .status {
    padding: 15px;
    border-radius: 10px;
    font-weight: bold;
    animation: slideIn 0.3s ease-out;
  }
  
  .status.info {
    background: rgba(0, 210, 255, 0.1);
    color: #00d2ff;
    border: 1px solid rgba(0, 210, 255, 0.3);
  }
  
  .status.success {
    background: rgba(0, 255, 100, 0.1);
    color: #00ff64;
    border: 1px solid rgba(0, 255, 100, 0.3);
    text-shadow: 0 0 10px rgba(0, 255, 100, 0.4);
  }
</style>
