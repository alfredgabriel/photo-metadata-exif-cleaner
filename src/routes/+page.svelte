<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { _, locale } from 'svelte-i18n';

  let processing = false;
  let statusMessage = '';
  let statusType = 'info';
  let removeGps = true;
  let removeCamera = true;
  let removeDate = true;

  type MetaTag = { key: string; value: string };
  type FileEntry = { name: string; path: string; meta: MetaTag[]; loadingMeta: boolean };
  let selectedFiles: FileEntry[] = [];
  let isCleaned = false;
  let dragActive = false;

  const languages = [
    { code: 'en', label: 'English' },
    { code: 'es', label: 'Español' }
  ];

  function setLocale(code: string) { locale.set(code); }

  // Returns human-readable label for an EXIF key using i18n
  function exifLabel(key: string): string {
    const label = $_('exif_labels.' + key);
    // If translation not found, fallback to formatted key
    if (label.startsWith('exif_labels.')) {
      return key.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
    }
    return label;
  }

  // GPS keys for highlight
  const gpsKeys = ['gps_latitude', 'gps_longitude', 'gps_altitude'];
  const cameraKeys = ['make', 'model', 'lens_model', 'focal_length', 'aperture', 'exposure_time', 'iso'];
  const dateKeys = ['datetime', 'datetime_original'];

  function tagColor(key: string): string {
    if (gpsKeys.includes(key)) return 'tag-red';
    if (cameraKeys.includes(key)) return 'tag-blue';
    if (dateKeys.includes(key)) return 'tag-orange';
    return 'tag-gray';
  }

  async function loadMetaForFile(entry: FileEntry) {
    entry.loadingMeta = true;
    selectedFiles = selectedFiles;
    try {
      const tags: MetaTag[] = await invoke('read_exif', { filePath: entry.path });
      entry.meta = tags;
    } catch (e) {
      entry.meta = [{ key: 'error', value: String(e) }];
    } finally {
      entry.loadingMeta = false;
      selectedFiles = selectedFiles;
    }
  }

  async function handleBrowse() {
    try {
      const selected = await open({
        multiple: true,
        filters: [{ name: 'Images', extensions: ['jpg', 'jpeg'] }]
      });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      statusMessage = '';
      isCleaned = false;
      selectedFiles = paths.map(p => ({
        name: p.split(/[\\/]/).pop() || p,
        path: p,
        meta: [],
        loadingMeta: true
      }));
      // Read metadata for all files
      for (const entry of selectedFiles) {
        await loadMetaForFile(entry);
      }
    } catch (e) {
      console.error(e);
    }
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    dragActive = false;
    const files = Array.from(e.dataTransfer?.files || []);
    if (files.length === 0) return;
    statusMessage = '';
    isCleaned = false;
    selectedFiles = files.map(f => ({
      name: f.name,
      path: (f as any).path || '',
      meta: [],
      loadingMeta: true
    }));
    for (const entry of selectedFiles) {
      if (entry.path) await loadMetaForFile(entry);
      else {
        entry.meta = [{ key: 'error', value: 'No path available. Use Browse Files.' }];
        entry.loadingMeta = false;
      }
    }
    selectedFiles = selectedFiles;
  }

  function handleDragOver(e: DragEvent) { e.preventDefault(); dragActive = true; }
  function handleDragLeave() { dragActive = false; }

  async function cleanImages() {
    processing = true;
    statusMessage = $_('cleaning');
    statusType = 'info';
    const filePaths = selectedFiles.map(s => s.path).filter(p => !!p);
    if (!filePaths.length) {
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

  function reset() { selectedFiles = []; isCleaned = false; statusMessage = ''; }
</script>

<div class="app">
  <!-- Sidebar -->
  <aside class="sidebar">
    <div class="brand">
      <div class="logo-wrap">
        <svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
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

    <div class="section">
      <span class="section-label">{$_('language')}</span>
      <div class="lang-btns">
        {#each languages as lang}
          <button class="lang-btn {$locale === lang.code ? 'active' : ''}" on:click={() => setLocale(lang.code)}>
            {lang.label}
          </button>
        {/each}
      </div>
    </div>

    <div class="section">
      <span class="section-label">{$_('options')}</span>
      <label class="toggle">
        <span class="toggle-icon">📍</span>
        <span class="label-text">{$_('remove_gps')}</span>
        <div class="switch"><input type="checkbox" bind:checked={removeGps}><span class="slider"></span></div>
      </label>
      <label class="toggle">
        <span class="toggle-icon">📷</span>
        <span class="label-text">{$_('remove_camera')}</span>
        <div class="switch"><input type="checkbox" bind:checked={removeCamera}><span class="slider"></span></div>
      </label>
      <label class="toggle">
        <span class="toggle-icon">🗓️</span>
        <span class="label-text">{$_('remove_date')}</span>
        <div class="switch"><input type="checkbox" bind:checked={removeDate}><span class="slider"></span></div>
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
              <polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/>
            </svg>
          </div>
          <p class="drop-title">{$_('drag')}</p>
          <p class="drop-sub">{$_('or')}</p>
          <button class="btn btn-secondary" on:click|stopPropagation={handleBrowse}>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
            {$_('browse')}
          </button>
          <p class="drop-hint">JPG · JPEG</p>
        </div>
      </div>

    {:else}
      <div class="content">
        <div class="content-header">
          <div class="content-title">
            <h2>{selectedFiles.length} {selectedFiles.length === 1 ? $_('file_singular') : $_('file_plural')} {$_('selected')}</h2>
            <span class="file-count-badge">{selectedFiles.reduce((n, f) => n + f.meta.filter(m => gpsKeys.includes(m.key) || cameraKeys.includes(m.key) || dateKeys.includes(m.key)).length, 0)} {$_('meta_section')}</span>
          </div>
          {#if !isCleaned}
            <button class="btn btn-ghost" on:click={reset}>{$_('clear')}</button>
          {/if}
        </div>

        <div class="file-list">
          {#each selectedFiles as f}
            <div class="file-card {isCleaned ? 'cleaned' : ''}">
              <div class="file-card-header">
                <div class="file-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                    <rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/>
                  </svg>
                </div>
                <div class="file-header-info">
                  <p class="file-name">{f.name}</p>
                  {#if f.meta.find(m => m.key === 'file_size')}
                    <p class="file-size">{f.meta.find(m => m.key === 'file_size')?.value}</p>
                  {/if}
                </div>
                {#if isCleaned}
                  <div class="badge-done">✓</div>
                {:else if f.loadingMeta}
                  <span class="loading-dots">···</span>
                {/if}
              </div>

              {#if !f.loadingMeta && f.meta.length > 0}
                <div class="meta-tags">
                  {#each f.meta.filter(m => m.key !== 'file_size' && m.key !== 'has_exif') as m}
                    <span class="tag {tagColor(m.key)}">
                      <span class="tag-key">{exifLabel(m.key)}</span>
                      <span class="tag-val">{m.value}</span>
                    </span>
                  {/each}
                  {#if f.meta.find(m => m.key === 'has_exif' && m.value === 'false') && f.meta.filter(m => m.key !== 'file_size' && m.key !== 'has_exif').length === 0}
                    <span class="tag tag-green"><span class="tag-key">{$_('no_exif')}</span></span>
                  {/if}
                </div>
              {:else if f.loadingMeta}
                <p class="loading-text">{$_('loading_meta')}</p>
              {/if}
            </div>
          {/each}
        </div>

        <div class="action-bar">
          {#if statusMessage}
            <p class="status-msg {statusType}">{statusMessage}</p>
          {:else}
            <span></span>
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
            <button class="btn btn-secondary" on:click={reset}>{$_('clean_another')}</button>
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
    background: #09090b; color: #e4e4e7; height: 100vh; overflow: hidden;
  }
  .app { display: flex; height: 100vh; }

  /* Sidebar */
  .sidebar {
    width: 260px; min-width: 260px;
    background: #111113; border-right: 1px solid #27272a;
    display: flex; flex-direction: column; padding: 24px 18px; gap: 24px;
    overflow-y: auto;
  }
  .brand { display: flex; align-items: center; gap: 12px; }
  .logo-wrap { width: 44px; height: 44px; flex-shrink: 0; }
  .logo-wrap svg { width: 100%; height: 100%; }
  .brand-text h1 { font-size: 0.95rem; font-weight: 700; color: #f4f4f5; line-height: 1.2; }
  .tagline { font-size: 0.7rem; color: #71717a; margin-top: 2px; line-height: 1.3; }
  .section { display: flex; flex-direction: column; gap: 8px; }
  .section-label { font-size: 0.65rem; font-weight: 600; text-transform: uppercase; letter-spacing: 0.1em; color: #52525b; }
  .lang-btns { display: flex; gap: 6px; }
  .lang-btn {
    flex: 1; padding: 6px 8px;
    background: #1c1c1f; border: 1px solid #3f3f46; color: #a1a1aa;
    border-radius: 6px; font-size: 0.8rem; font-weight: 500; cursor: pointer; transition: all 0.15s;
  }
  .lang-btn:hover { background: #27272a; color: #e4e4e7; }
  .lang-btn.active { background: #0ea5e9; border-color: #0ea5e9; color: white; }
  .toggle {
    display: flex; align-items: center; gap: 10px; padding: 8px 10px;
    border-radius: 8px; cursor: pointer; transition: background 0.15s; user-select: none;
  }
  .toggle:hover { background: #1c1c1f; }
  .toggle-icon { font-size: 0.95rem; }
  .label-text { flex: 1; font-size: 0.82rem; color: #d4d4d8; line-height: 1.3; }
  .switch { position: relative; width: 36px; height: 20px; flex-shrink: 0; }
  .switch input { opacity: 0; width: 0; height: 0; }
  .slider { position: absolute; inset: 0; background: #3f3f46; border-radius: 20px; transition: 0.2s; }
  .slider::before {
    content: ''; position: absolute; width: 14px; height: 14px;
    left: 3px; top: 3px; background: white; border-radius: 50%; transition: 0.2s;
    box-shadow: 0 1px 3px rgba(0,0,0,.4);
  }
  input:checked + .slider { background: #0ea5e9; }
  input:checked + .slider::before { transform: translateX(16px); }
  .sidebar-footer {
    margin-top: auto; display: flex; align-items: center; gap: 6px;
    font-size: 0.72rem; color: #52525b; padding-top: 16px; border-top: 1px solid #1c1c1f;
  }

  /* Main */
  .main {
    flex: 1; padding: 28px; display: flex; flex-direction: column;
    background: #09090b;
    background-image: radial-gradient(#1f1f22 1px, transparent 1px);
    background-size: 24px 24px;
    overflow: hidden;
  }

  /* Dropzone */
  .dropzone {
    flex: 1; border: 2px dashed #3f3f46; border-radius: 20px;
    display: flex; align-items: center; justify-content: center;
    background: rgba(17,17,19,0.8); backdrop-filter: blur(6px);
    transition: all 0.2s; cursor: pointer;
  }
  .dropzone:hover, .dropzone.drag-active { border-color: #0ea5e9; background: rgba(14,165,233,0.05); }
  .drop-inner { text-align: center; display: flex; flex-direction: column; align-items: center; gap: 10px; }
  .drop-icon {
    width: 68px; height: 68px; background: #1c1c1f; border-radius: 16px;
    display: flex; align-items: center; justify-content: center; color: #52525b; margin-bottom: 4px;
  }
  .drop-icon svg { width: 32px; height: 32px; }
  .drop-title { font-size: 1.15rem; font-weight: 600; color: #f4f4f5; }
  .drop-sub { font-size: 0.85rem; color: #71717a; }
  .drop-hint { font-size: 0.72rem; color: #52525b; margin-top: 4px; }

  /* Buttons */
  .btn {
    display: inline-flex; align-items: center; gap: 7px;
    padding: 9px 18px; border-radius: 8px;
    font-size: 0.875rem; font-weight: 600; cursor: pointer; border: none; transition: all 0.15s;
  }
  .btn-primary { background: #0ea5e9; color: white; }
  .btn-primary:hover:not(:disabled) { background: #0284c7; }
  .btn-primary:disabled { background: #27272a; color: #71717a; cursor: not-allowed; }
  .btn-secondary { background: #1c1c1f; color: #e4e4e7; border: 1px solid #3f3f46; }
  .btn-secondary:hover { background: #27272a; }
  .btn-ghost { background: transparent; color: #71717a; border: 1px solid transparent; font-size: 0.8rem; padding: 6px 12px; }
  .btn-ghost:hover { color: #e4e4e7; border-color: #3f3f46; }

  /* Content */
  .content { display: flex; flex-direction: column; gap: 16px; height: 100%; }
  .content-header { display: flex; align-items: center; justify-content: space-between; }
  .content-title { display: flex; align-items: center; gap: 10px; }
  .content-title h2 { font-size: 0.95rem; font-weight: 600; color: #f4f4f5; }
  .file-count-badge {
    font-size: 0.72rem; background: #1c1c1f; border: 1px solid #3f3f46;
    color: #a1a1aa; padding: 2px 8px; border-radius: 99px;
  }

  /* File list */
  .file-list { display: flex; flex-direction: column; gap: 10px; overflow-y: auto; flex: 1; }
  .file-card {
    background: #111113; border: 1px solid #27272a; border-radius: 12px;
    padding: 14px 16px; display: flex; flex-direction: column; gap: 10px;
    transition: border-color 0.15s;
  }
  .file-card:hover { border-color: #3f3f46; }
  .file-card.cleaned { border-color: #166534; background: #0a1f0a; }

  .file-card-header { display: flex; align-items: center; gap: 12px; }
  .file-icon {
    width: 40px; height: 40px; flex-shrink: 0;
    background: #1c1c1f; border-radius: 8px;
    display: flex; align-items: center; justify-content: center; color: #52525b;
  }
  .file-icon svg { width: 20px; height: 20px; }
  .file-header-info { flex: 1; min-width: 0; }
  .file-name { font-size: 0.875rem; font-weight: 500; color: #f4f4f5; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .file-size { font-size: 0.75rem; color: #71717a; margin-top: 2px; }

  .badge-done {
    width: 26px; height: 26px; border-radius: 50%;
    background: #052e16; border: 1px solid #166534;
    color: #4ade80; display: flex; align-items: center; justify-content: center; font-size: 0.8rem; font-weight: 700;
  }
  .loading-dots { color: #71717a; font-size: 1.2rem; letter-spacing: 2px; animation: blink 1s infinite; }
  @keyframes blink { 0%,100%{opacity:1} 50%{opacity:0.3} }
  .loading-text { font-size: 0.78rem; color: #71717a; font-style: italic; }

  /* Tags */
  .meta-tags { display: flex; flex-wrap: wrap; gap: 6px; }
  .tag {
    display: inline-flex; align-items: center; gap: 5px;
    font-size: 0.72rem; padding: 3px 8px; border-radius: 6px;
  }
  .tag-key { font-weight: 600; opacity: 0.8; }
  .tag-val { }
  .tag-red { background: rgba(239,68,68,0.12); border: 1px solid rgba(239,68,68,0.25); color: #fca5a5; }
  .tag-blue { background: rgba(59,130,246,0.12); border: 1px solid rgba(59,130,246,0.25); color: #93c5fd; }
  .tag-orange { background: rgba(251,146,60,0.12); border: 1px solid rgba(251,146,60,0.25); color: #fdba74; }
  .tag-gray { background: #1c1c1f; border: 1px solid #3f3f46; color: #a1a1aa; }
  .tag-green { background: rgba(74,222,128,0.1); border: 1px solid rgba(74,222,128,0.2); color: #86efac; }

  /* Action bar */
  .action-bar {
    display: flex; align-items: center; justify-content: space-between; gap: 14px;
    background: #111113; border: 1px solid #27272a; border-radius: 12px; padding: 12px 18px;
    flex-shrink: 0;
  }
  .status-msg { flex: 1; font-size: 0.85rem; }
  .status-msg.success { color: #4ade80; }
  .status-msg.error { color: #f87171; }
  .status-msg.info { color: #38bdf8; }
  .spinner {
    display: inline-block; width: 13px; height: 13px;
    border: 2px solid rgba(255,255,255,0.3); border-top-color: white;
    border-radius: 50%; animation: spin 0.6s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
