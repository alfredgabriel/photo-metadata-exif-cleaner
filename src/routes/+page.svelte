<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { _, locale } from 'svelte-i18n';
  import * as ExifReader from 'exifreader';

  let processing = false;
  let statusMessage = '';
  let statusType = 'info';
  let removeGps = true;
  let removeCamera = true;
  let removeDate = true;

  type FileEntry = { name: string; path: string; preview: string; meta: { key: string; value: string }[] };
  let selectedFiles: FileEntry[] = [];
  let isCleaned = false;
  let dragActive = false;

  const languages = [
    { code: 'en', label: 'English' },
    { code: 'es', label: 'Español' }
  ];

  function setLocale(code: string) {
    locale.set(code);
  }

  async function loadFilesFromPaths(paths: string[]) {
    statusMessage = '';
    isCleaned = false;
    selectedFiles = paths.map(p => ({
      name: p.split(/[\\/]/).pop() || p,
      path: p,
      preview: '',
      meta: [{ key: 'File', value: p.split(/[\\/]/).pop() || p }]
    }));
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    dragActive = false;
    const files = Array.from(e.dataTransfer?.files || []);
    if (files.length === 0) return;
    statusMessage = '';
    isCleaned = false;
    selectedFiles = [];
    for (const file of files) {
      const path = (file as any).path || '';
      const preview = URL.createObjectURL(file);
      let meta: { key: string; value: string }[] = [];
      try {
        const tags = await ExifReader.load(file);
        if (tags['GPSLatitude']) meta.push({ key: $_('gps'), value: $_('detected_val') });
        if (tags['Model']) meta.push({ key: $_('camera'), value: tags['Model'].description });
        if (tags['DateTimeOriginal']) meta.push({ key: $_('date_taken'), value: tags['DateTimeOriginal'].description });
        if (tags['LensModel']) meta.push({ key: $_('lens'), value: tags['LensModel'].description });
        if (meta.length === 0) meta.push({ key: $_('status_clean'), value: $_('clean_val') });
      } catch {
        meta = [{ key: 'Info', value: $_('no_exif') }];
      }
      selectedFiles = [...selectedFiles, { name: file.name, path, preview, meta }];
    }
  }

  function handleDragOver(e: DragEvent) { e.preventDefault(); dragActive = true; }
  function handleDragLeave() { dragActive = false; }

  async function handleBrowse() {
    try {
      const selected = await open({
        multiple: true,
        filters: [{ name: 'Images', extensions: ['jpg', 'jpeg', 'png'] }]
      });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      await loadFilesFromPaths(paths);
    } catch (e) {
      console.error(e);
    }
  }

  async function cleanImages() {
    processing = true;
    statusMessage = $_('cleaning');
    statusType = 'info';
    const filePaths = selectedFiles.map(s => s.path).filter(p => p);
    if (filePaths.length === 0) {
      statusMessage = $_('error_no_path');
      statusType = 'error';
      processing = false;
      return;
    }
    try {
      await invoke('clean_exif', { filePaths });
      statusMessage = $_('cleaned_success');
      statusType = 'success';
      isCleaned = true;
    } catch (err) {
      statusMessage = $_('error') + ': ' + err;
      statusType = 'error';
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

<div class="app">
  <!-- Sidebar -->
  <aside class="sidebar">
    <div class="brand">
      <div class="logo-wrap">
        <svg viewBox="0 0 100 100" class="logo-svg" xmlns="http://www.w3.org/2000/svg">
          <rect x="8" y="32" width="84" height="55" rx="10" ry="10" fill="#0ea5e9"/>
          <path d="M35 32 L38 20 L62 20 L65 32 Z" fill="#0ea5e9"/>
          <circle cx="50" cy="58" r="20" fill="#0c4a6e"/>
          <circle cx="50" cy="58" r="14" fill="#0369a1"/>
          <circle cx="50" cy="58" r="8" fill="#7dd3fc"/>
          <circle cx="47" cy="54" r="2.5" fill="white" opacity="0.6"/>
          <circle cx="78" cy="42" r="4" fill="white" opacity="0.85"/>
          <polygon points="76,16 78,21 83,22 78,23 76,28 74,23 69,22 74,21" fill="white" opacity="0.9"/>
        </svg>
      </div>
      <div class="brand-text">
        <h1>{$_('title')}</h1>
        <p class="tagline">{$_('subtitle')}</p>
      </div>
    </div>

    <!-- Language Switcher -->
    <div class="section">
      <span class="section-label">{$_('language')}</span>
      <div class="lang-btns">
        {#each languages as lang}
          <button
            class="lang-btn {$locale === lang.code ? 'active' : ''}"
            on:click={() => setLocale(lang.code)}
          >{lang.label}</button>
        {/each}
      </div>
    </div>

    <!-- Options -->
    <div class="section">
      <span class="section-label">{$_('options')}</span>
      <label class="toggle">
        <span class="toggle-icon">📍</span>
        <span class="label-text">{$_('remove_gps')}</span>
        <div class="switch">
          <input type="checkbox" bind:checked={removeGps}>
          <span class="slider"></span>
        </div>
      </label>
      <label class="toggle">
        <span class="toggle-icon">📷</span>
        <span class="label-text">{$_('remove_camera')}</span>
        <div class="switch">
          <input type="checkbox" bind:checked={removeCamera}>
          <span class="slider"></span>
        </div>
      </label>
      <label class="toggle">
        <span class="toggle-icon">🗓️</span>
        <span class="label-text">{$_('remove_date')}</span>
        <div class="switch">
          <input type="checkbox" bind:checked={removeDate}>
          <span class="slider"></span>
        </div>
      </label>
    </div>

    <div class="sidebar-footer">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
      {$_('local_processing')}
    </div>
  </aside>

  <!-- Main -->
  <main class="main">
    {#if selectedFiles.length === 0}
      <!-- Drop Zone -->
      <div
        class="dropzone {dragActive ? 'drag-active' : ''}"
        on:drop={handleDrop}
        on:dragover={handleDragOver}
        on:dragleave={handleDragLeave}
        role="button"
        tabindex="0"
      >
        <div class="drop-inner">
          <div class="drop-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="17 8 12 3 7 8"/>
              <line x1="12" y1="3" x2="12" y2="15"/>
            </svg>
          </div>
          <p class="drop-title">{$_('drag')}</p>
          <p class="drop-sub">{$_('or')}</p>
          <button class="btn btn-secondary" on:click|stopPropagation={handleBrowse}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
            {$_('browse')}
          </button>
          <p class="drop-hint">JPG, JPEG, PNG</p>
        </div>
      </div>
    {:else}
      <!-- File List -->
      <div class="content">
        <div class="content-header">
          <h2>{selectedFiles.length} {selectedFiles.length === 1 ? $_('file_singular') : $_('file_plural')} {$_('selected')}</h2>
          {#if !isCleaned}
            <button class="btn btn-ghost" on:click={reset}>{$_('clear')}</button>
          {/if}
        </div>

        <div class="file-list">
          {#each selectedFiles as f}
            <div class="file-card">
              <div class="file-thumb">
                {#if f.preview}
                  <img src={f.preview} alt={f.name} />
                {:else}
                  <div class="thumb-placeholder">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                      <rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/>
                    </svg>
                  </div>
                {/if}
              </div>
              <div class="file-info">
                <p class="file-name">{f.name}</p>
                <div class="tags">
                  {#each f.meta as m}
                    <span class="tag"><strong>{m.key}:</strong> {m.value}</span>
                  {/each}
                </div>
              </div>
              {#if isCleaned}
                <div class="badge-done">✓</div>
              {/if}
            </div>
          {/each}
        </div>

        <!-- Action Bar -->
        <div class="action-bar">
          {#if statusMessage}
            <p class="status-msg {statusType}">{statusMessage}</p>
          {/if}
          {#if !isCleaned}
            <button class="btn btn-primary" on:click={cleanImages} disabled={processing}>
              {#if processing}
                <span class="spinner"></span> {$_('cleaning')}
              {:else}
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
                {$_('clean')}
              {/if}
            </button>
          {:else}
            <button class="btn btn-secondary" on:click={reset}>
              {$_('clean_another')}
            </button>
          {/if}
        </div>
      </div>
    {/if}
  </main>
</div>

<style>
  :global(*, *::before, *::after) { box-sizing: border-box; margin: 0; padding: 0; }

  :global(body) {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: #09090b;
    color: #e4e4e7;
    height: 100vh;
    overflow: hidden;
  }

  .app {
    display: flex;
    height: 100vh;
    width: 100%;
  }

  /* ---- Sidebar ---- */
  .sidebar {
    width: 280px;
    min-width: 280px;
    background: #111113;
    border-right: 1px solid #27272a;
    display: flex;
    flex-direction: column;
    padding: 28px 20px;
    gap: 28px;
    overflow-y: auto;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .logo-wrap {
    width: 52px;
    height: 52px;
    flex-shrink: 0;
  }

  .logo-svg {
    width: 100%;
    height: 100%;
  }

  .brand-text h1 {
    font-size: 1.05rem;
    font-weight: 700;
    color: #f4f4f5;
    line-height: 1.2;
  }

  .tagline {
    font-size: 0.75rem;
    color: #71717a;
    margin-top: 2px;
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .section-label {
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: #52525b;
  }

  /* Language */
  .lang-btns {
    display: flex;
    gap: 8px;
  }

  .lang-btn {
    flex: 1;
    padding: 6px 8px;
    background: #1c1c1f;
    border: 1px solid #3f3f46;
    color: #a1a1aa;
    border-radius: 6px;
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
  }

  .lang-btn:hover { background: #27272a; color: #e4e4e7; }
  .lang-btn.active {
    background: #0ea5e9;
    border-color: #0ea5e9;
    color: white;
  }

  /* Toggles */
  .toggle {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.15s;
    user-select: none;
  }
  .toggle:hover { background: #1c1c1f; }
  .toggle-icon { font-size: 1rem; }
  .label-text { flex: 1; font-size: 0.875rem; color: #d4d4d8; }

  .switch { position: relative; width: 38px; height: 22px; flex-shrink: 0; }
  .switch input { opacity: 0; width: 0; height: 0; }
  .slider {
    position: absolute; inset: 0;
    background: #3f3f46;
    border-radius: 22px;
    transition: 0.2s;
  }
  .slider::before {
    content: '';
    position: absolute;
    width: 16px; height: 16px;
    left: 3px; top: 3px;
    background: white;
    border-radius: 50%;
    transition: 0.2s;
    box-shadow: 0 1px 3px rgba(0,0,0,.4);
  }
  input:checked + .slider { background: #0ea5e9; }
  input:checked + .slider::before { transform: translateX(16px); }

  .sidebar-footer {
    margin-top: auto;
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.75rem;
    color: #52525b;
    padding-top: 20px;
    border-top: 1px solid #1c1c1f;
  }

  /* ---- Main ---- */
  .main {
    flex: 1;
    padding: 32px;
    display: flex;
    flex-direction: column;
    background: #09090b;
    background-image: radial-gradient(#27272a 1px, transparent 1px);
    background-size: 28px 28px;
    overflow-y: auto;
  }

  /* Drop Zone */
  .dropzone {
    flex: 1;
    min-height: 400px;
    border: 2px dashed #3f3f46;
    border-radius: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(17,17,19,0.7);
    backdrop-filter: blur(6px);
    transition: border-color 0.2s, background 0.2s;
    cursor: pointer;
  }
  .dropzone:hover, .dropzone.drag-active {
    border-color: #0ea5e9;
    background: rgba(14,165,233,0.04);
  }

  .drop-inner { text-align: center; display: flex; flex-direction: column; align-items: center; gap: 12px; }

  .drop-icon {
    width: 72px; height: 72px;
    background: #1c1c1f;
    border-radius: 16px;
    display: flex; align-items: center; justify-content: center;
    color: #52525b;
    margin-bottom: 4px;
  }
  .drop-icon svg { width: 36px; height: 36px; }

  .drop-title { font-size: 1.2rem; font-weight: 600; color: #f4f4f5; }
  .drop-sub { font-size: 0.875rem; color: #71717a; }
  .drop-hint { font-size: 0.75rem; color: #52525b; margin-top: 4px; }

  /* Buttons */
  .btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 10px 20px;
    border-radius: 8px;
    font-size: 0.9rem;
    font-weight: 600;
    cursor: pointer;
    border: none;
    transition: all 0.15s;
  }
  .btn-primary { background: #0ea5e9; color: white; }
  .btn-primary:hover:not(:disabled) { background: #0284c7; }
  .btn-primary:disabled { background: #27272a; color: #71717a; cursor: not-allowed; }
  .btn-secondary { background: #1c1c1f; color: #e4e4e7; border: 1px solid #3f3f46; }
  .btn-secondary:hover { background: #27272a; }
  .btn-ghost { background: transparent; color: #71717a; border: 1px solid transparent; }
  .btn-ghost:hover { color: #e4e4e7; border-color: #3f3f46; }

  /* Content */
  .content { display: flex; flex-direction: column; gap: 20px; flex: 1; }

  .content-header {
    display: flex; align-items: center; justify-content: space-between;
  }
  .content-header h2 { font-size: 1rem; font-weight: 600; color: #f4f4f5; }

  /* File List */
  .file-list { display: flex; flex-direction: column; gap: 12px; overflow-y: auto; flex: 1; }

  .file-card {
    display: flex; align-items: center; gap: 14px;
    background: #111113;
    border: 1px solid #27272a;
    border-radius: 12px;
    padding: 14px 16px;
    transition: border-color 0.15s;
  }
  .file-card:hover { border-color: #3f3f46; }

  .file-thumb {
    width: 56px; height: 56px; flex-shrink: 0;
    border-radius: 8px; overflow: hidden; background: #1c1c1f;
  }
  .file-thumb img { width: 100%; height: 100%; object-fit: cover; }
  .thumb-placeholder {
    width: 100%; height: 100%;
    display: flex; align-items: center; justify-content: center;
    color: #52525b;
  }
  .thumb-placeholder svg { width: 24px; height: 24px; }

  .file-info { flex: 1; min-width: 0; }
  .file-name { font-size: 0.875rem; font-weight: 500; color: #e4e4e7; margin-bottom: 6px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

  .tags { display: flex; flex-wrap: wrap; gap: 6px; }
  .tag {
    font-size: 0.75rem; padding: 2px 8px;
    background: #1c1c1f; border: 1px solid #3f3f46;
    border-radius: 99px; color: #a1a1aa;
  }
  .tag strong { color: #e4e4e7; }

  .badge-done {
    width: 28px; height: 28px; border-radius: 50%;
    background: #052e16; border: 1px solid #166534;
    color: #4ade80; display: flex; align-items: center; justify-content: center;
    font-size: 0.85rem; font-weight: 700;
  }

  /* Action Bar */
  .action-bar {
    display: flex; align-items: center; justify-content: flex-end; gap: 14px;
    background: #111113; border: 1px solid #27272a;
    border-radius: 12px; padding: 14px 20px;
  }

  .status-msg { flex: 1; font-size: 0.875rem; }
  .status-msg.success { color: #4ade80; }
  .status-msg.error { color: #f87171; }
  .status-msg.info { color: #38bdf8; }

  .spinner {
    display: inline-block; width: 14px; height: 14px;
    border: 2px solid rgba(255,255,255,0.3);
    border-top-color: white; border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
