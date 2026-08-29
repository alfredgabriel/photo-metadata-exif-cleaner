<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { _, locale } from 'svelte-i18n';

  // --- Types ---
  type ExifTag = { key: string; value: string };
  type FileInfo = { tags: ExifTag[]; file_size: string; preview_b64: string; mime: string; format: string };
  type FileEntry = { name: string; path: string; info: FileInfo | null; loading: boolean; error: string | null };

  // --- State ---
  let files: FileEntry[] = [];
  let isCleaned = false;
  let outputFolder = '';
  let processing = false;
  let statusMsg = '';
  let statusType: 'info' | 'success' | 'error' = 'info';
  let dragActive = false;
  let removeGps = true;
  let removeCamera = true;
  let removeDate = true;

  const LANGS = [{ code: 'en', label: 'EN' }, { code: 'es', label: 'ES' }];
  const EXTS = ['jpg', 'jpeg', 'png', 'webp', 'tif', 'tiff'];

  // --- Tag categorisation ---
  function tagCategory(key: string): 'gps' | 'camera' | 'date' | 'other' {
    const k = key.toLowerCase();
    if (k.startsWith('gps')) return 'gps';
    if (['make','model','lensmodel','lensmake','focallength','fnumber','exposuretime','photographicsensitivity',
         'isospeedratings','flash','whitebalance','exposuremode','meteringmode','scenecapturetype',
         'exposurebias','maxaperturevalue','subjectdistance'].includes(k)) return 'camera';
    if (['datetime','datetimeoriginal','datetimedigitized','subsectime'].some(d => k.startsWith(d))) return 'date';
    return 'other';
  }

  // Decide whether a tag should be shown based on user options
  function shouldShow(key: string): boolean {
    const cat = tagCategory(key);
    if (cat === 'gps' && !removeGps) return false;
    if (cat === 'camera' && !removeCamera) return false;
    if (cat === 'date' && !removeDate) return false;
    return true;
  }

  function tagClass(key: string): string {
    switch (tagCategory(key)) {
      case 'gps':    return 'tag-red';
      case 'camera': return 'tag-blue';
      case 'date':   return 'tag-orange';
      default:       return 'tag-gray';
    }
  }

  // --- Load EXIF for one entry ---
  async function loadInfo(path: string): Promise<FileInfo | null> {
    try {
      return await invoke<FileInfo>('read_exif', { filePath: path });
    } catch {
      return null;
    }
  }

  // --- Add paths ---
  async function addPaths(paths: string[]) {
    isCleaned = false;
    outputFolder = '';
    statusMsg = '';
    files = paths.map(p => ({
      name: p.replace(/\\/g, '/').split('/').pop() || p,
      path: p,
      info: null,
      loading: true,
      error: null,
    }));

    for (let i = 0; i < files.length; i++) {
      const info = await loadInfo(files[i].path);
      files = files.map((f, idx) => idx === i ? { ...f, info, loading: false } : f);
    }
  }

  // --- Browse ---
  async function handleBrowse() {
    try {
      const sel = await open({ multiple: true, filters: [{ name: 'Images', extensions: EXTS }] });
      if (!sel) return;
      await addPaths(Array.isArray(sel) ? sel : [sel]);
    } catch (e) { console.error(e); }
  }

  // --- Drag & Drop ---
  function handleDragOver(e: DragEvent) { e.preventDefault(); dragActive = true; }
  function handleDragLeave() { dragActive = false; }
  async function handleDrop(e: DragEvent) {
    e.preventDefault(); dragActive = false;
    const dropped = Array.from(e.dataTransfer?.files || []);
    const paths = dropped.map(f => (f as any).path).filter(Boolean);
    if (paths.length) await addPaths(paths);
  }

  // --- Clean ---
  async function cleanImages() {
    processing = true;
    statusMsg = $_('cleaning');
    statusType = 'info';
    const paths = files.map(f => f.path).filter(Boolean);
    try {
      const dir = await invoke<string>('clean_exif', { filePaths: paths });
      outputFolder = dir;
      isCleaned = true;
      statusMsg = $_('cleaned_success');
      statusType = 'success';
    } catch (err) {
      statusMsg = $_('error') + ': ' + err;
      statusType = 'error';
    } finally { processing = false; }
  }

  // --- Open folder ---
  async function openFolder() {
    if (outputFolder) await invoke('open_folder', { path: outputFolder });
  }

  function reset() { files = []; isCleaned = false; statusMsg = ''; outputFolder = ''; }

  // --- Computed stats ---
  $: totalTags = files.reduce((n, f) => n + (f.info?.tags.length ?? 0), 0);
  $: gpsFiles = files.filter(f => f.info?.tags.some(t => t.key.toLowerCase().startsWith('gps'))).length;
</script>

<div class="app">
  <!-- SIDEBAR -->
  <aside class="sidebar">
    <div class="brand">
      <svg class="logo" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
        <rect x="8" y="32" width="84" height="55" rx="10" fill="#0ea5e9"/>
        <path d="M35 32 L38 20 L62 20 L65 32 Z" fill="#0ea5e9"/>
        <circle cx="50" cy="58" r="20" fill="#0c4a6e"/>
        <circle cx="50" cy="58" r="14" fill="#0369a1"/>
        <circle cx="50" cy="58" r="8" fill="#7dd3fc"/>
        <circle cx="47" cy="54" r="2.5" fill="white" opacity="0.6"/>
        <circle cx="78" cy="42" r="4" fill="white" opacity="0.85"/>
        <polygon points="76,16 78,21 83,22 78,23 76,28 74,23 69,22 74,21" fill="white" opacity="0.9"/>
      </svg>
      <div>
        <div class="app-name">{$_('title')}</div>
        <div class="app-sub">{$_('subtitle')}</div>
      </div>
    </div>

    <!-- Language -->
    <div class="section">
      <div class="section-label">{$_('language')}</div>
      <div class="row gap-6">
        {#each LANGS as l}
          <button class="lang-btn {$locale?.startsWith(l.code) ? 'active' : ''}" on:click={() => locale.set(l.code)}>{l.label}</button>
        {/each}
      </div>
    </div>

    <!-- Options -->
    <div class="section">
      <div class="section-label">{$_('options')}</div>
      <label class="opt-row">
        <span class="opt-dot red"></span>
        <span class="opt-text">{$_('remove_gps')}</span>
        <div class="sw"><input type="checkbox" bind:checked={removeGps}><span class="sw-track"></span></div>
      </label>
      <label class="opt-row">
        <span class="opt-dot blue"></span>
        <span class="opt-text">{$_('remove_camera')}</span>
        <div class="sw"><input type="checkbox" bind:checked={removeCamera}><span class="sw-track"></span></div>
      </label>
      <label class="opt-row">
        <span class="opt-dot orange"></span>
        <span class="opt-text">{$_('remove_date')}</span>
        <div class="sw"><input type="checkbox" bind:checked={removeDate}><span class="sw-track"></span></div>
      </label>
    </div>

    <!-- Stats (when files loaded) -->
    {#if files.length > 0 && !isCleaned}
      <div class="section">
        <div class="section-label">{$_('summary')}</div>
        <div class="stat-grid">
          <div class="stat"><span class="stat-n">{files.length}</span><span class="stat-l">{$_('files_label')}</span></div>
          <div class="stat"><span class="stat-n">{totalTags}</span><span class="stat-l">{$_('tags_label')}</span></div>
          <div class="stat"><span class="stat-n red-txt">{gpsFiles}</span><span class="stat-l">{$_('gps_label')}</span></div>
        </div>
      </div>
    {/if}

    <div class="sidebar-footer">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
      {$_('local_processing')}
    </div>
  </aside>

  <!-- MAIN -->
  <main class="main">
    {#if files.length === 0}
      <!-- DROP ZONE -->
      <div
        class="dropzone {dragActive ? 'active' : ''}"
        on:drop={handleDrop}
        on:dragover={handleDragOver}
        on:dragleave={handleDragLeave}
        role="button" tabindex="0"
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
          <button class="btn primary" on:click|stopPropagation={handleBrowse}>
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
            {$_('browse')}
          </button>
          <p class="drop-hint">JPG · JPEG · PNG · WEBP · TIFF</p>
        </div>
      </div>

    {:else}
      <!-- FILE LIST -->
      <div class="content">
        <div class="top-bar">
          <div class="top-left">
            <span class="badge">{files.length} {files.length === 1 ? $_('file_singular') : $_('file_plural')}</span>
            {#if !isCleaned}
              <button class="btn ghost sm" on:click={handleBrowse}>{$_('add_more')}</button>
            {/if}
          </div>
          {#if !isCleaned}
            <button class="btn ghost sm" on:click={reset}>{$_('clear')}</button>
          {/if}
        </div>

        <div class="file-list">
          {#each files as f (f.path)}
            <div class="file-card {isCleaned ? 'done' : ''}">
              <!-- Thumbnail -->
              <div class="thumb-wrap">
                {#if f.info?.preview_b64}
                  <img
                    src="data:{f.info.mime};base64,{f.info.preview_b64}"
                    alt={f.name}
                    class="thumb"
                  />
                {:else if f.loading}
                  <div class="thumb-ph"><span class="spin"></span></div>
                {:else}
                  <div class="thumb-ph">
                    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                      <rect x="3" y="3" width="18" height="18" rx="2"/>
                      <circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/>
                    </svg>
                  </div>
                {/if}
                {#if isCleaned}<div class="done-badge">✓</div>{/if}
              </div>

              <!-- Info -->
              <div class="file-body">
                <div class="file-top">
                  <span class="file-name">{f.name}</span>
                  {#if f.info?.file_size}<span class="file-size">{f.info.file_size}</span>{/if}
                </div>

                {#if f.loading}
                  <p class="loading-txt">{$_('loading_meta')}</p>
                {:else if f.info && f.info.tags.length > 0}
                  <div class="tags-wrap">
                    {#each f.info.tags.filter(t => shouldShow(t.key)) as t}
                      <span class="tag {tagClass(t.key)}" title="{t.key}: {t.value}">
                        <span class="tk">{t.key}</span>
                        <span class="tv">{t.value.length > 30 ? t.value.slice(0, 30) + '…' : t.value}</span>
                      </span>
                    {/each}
                    {#if f.info.tags.filter(t => shouldShow(t.key)).length === 0}
                      <span class="tag tag-green"><span class="tk">{$_('no_exif')}</span></span>
                    {/if}
                  </div>
                {:else}
                  <span class="tag tag-green"><span class="tk">{$_('no_exif')}</span></span>
                {/if}
              </div>
            </div>
          {/each}
        </div>

        <!-- ACTION BAR -->
        <div class="action-bar">
          <div class="status-area">
            {#if statusMsg}
              <p class="status {statusType}">{statusMsg}</p>
            {/if}
          </div>
          <div class="action-btns">
            {#if isCleaned}
              <button class="btn secondary" on:click={openFolder}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
                </svg>
                {$_('open_folder')}
              </button>
              <button class="btn primary" on:click={reset}>{$_('clean_another')}</button>
            {:else}
              <button class="btn primary" on:click={cleanImages} disabled={processing}>
                {#if processing}
                  <span class="spin white"></span> {$_('cleaning')}
                {:else}
                  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
                  {$_('clean')}
                {/if}
              </button>
            {/if}
          </div>
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

  /* ===================== SIDEBAR ===================== */
  .sidebar {
    width: 256px; min-width: 256px;
    background: #111113; border-right: 1px solid #1e1e21;
    display: flex; flex-direction: column;
    padding: 20px 16px; gap: 22px; overflow-y: auto;
  }

  .brand { display: flex; align-items: center; gap: 12px; }
  .logo { width: 40px; height: 40px; flex-shrink: 0; }
  .app-name { font-size: 0.9rem; font-weight: 700; color: #f4f4f5; line-height: 1.2; }
  .app-sub { font-size: 0.68rem; color: #71717a; margin-top: 2px; line-height: 1.3; }

  .section { display: flex; flex-direction: column; gap: 8px; }
  .section-label {
    font-size: 0.62rem; font-weight: 700; text-transform: uppercase;
    letter-spacing: 0.1em; color: #52525b;
  }

  .row { display: flex; }
  .gap-6 { gap: 6px; }

  .lang-btn {
    flex: 1; padding: 5px; background: #1a1a1d; border: 1px solid #3f3f46;
    color: #71717a; border-radius: 6px; font-size: 0.75rem; font-weight: 600;
    cursor: pointer; transition: all 0.15s;
  }
  .lang-btn:hover { color: #e4e4e7; border-color: #52525b; }
  .lang-btn.active { background: #0ea5e9; border-color: #0ea5e9; color: white; }

  /* Options */
  .opt-row {
    display: flex; align-items: center; gap: 10px; padding: 8px 10px;
    border-radius: 8px; cursor: pointer; transition: background 0.15s;
  }
  .opt-row:hover { background: #1a1a1d; }
  .opt-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
  .opt-dot.red { background: #ef4444; }
  .opt-dot.blue { background: #3b82f6; }
  .opt-dot.orange { background: #f97316; }
  .opt-text { flex: 1; font-size: 0.8rem; color: #d4d4d8; }

  .sw { position: relative; width: 34px; height: 18px; flex-shrink: 0; }
  .sw input { opacity: 0; width: 0; height: 0; }
  .sw-track {
    position: absolute; inset: 0; background: #3f3f46; border-radius: 18px; transition: 0.2s;
  }
  .sw-track::before {
    content: ''; position: absolute; width: 13px; height: 13px; left: 2.5px; top: 2.5px;
    background: white; border-radius: 50%; transition: 0.2s; box-shadow: 0 1px 3px rgba(0,0,0,.4);
  }
  .sw input:checked + .sw-track { background: #0ea5e9; }
  .sw input:checked + .sw-track::before { transform: translateX(16px); }

  /* Stats */
  .stat-grid { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 8px; }
  .stat {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    background: #1a1a1d; border: 1px solid #27272a; border-radius: 8px; padding: 10px 4px;
  }
  .stat-n { font-size: 1.3rem; font-weight: 700; color: #f4f4f5; line-height: 1; }
  .stat-l { font-size: 0.62rem; color: #71717a; margin-top: 2px; }
  .red-txt { color: #f87171 !important; }

  .sidebar-footer {
    margin-top: auto; display: flex; align-items: center; gap: 6px;
    font-size: 0.68rem; color: #52525b; padding-top: 16px; border-top: 1px solid #1a1a1d;
  }

  /* ===================== MAIN ===================== */
  .main {
    flex: 1; padding: 24px; display: flex; flex-direction: column;
    background: #09090b;
    background-image: radial-gradient(#1a1a1d 1px, transparent 1px);
    background-size: 24px 24px;
    overflow: hidden;
  }

  /* Drop Zone */
  .dropzone {
    flex: 1; border: 2px dashed #27272a; border-radius: 20px;
    display: flex; align-items: center; justify-content: center;
    background: rgba(17,17,19,0.8);
    transition: all 0.2s; cursor: pointer;
  }
  .dropzone.active, .dropzone:hover {
    border-color: #0ea5e9; background: rgba(14,165,233,0.04);
  }
  .drop-inner { text-align: center; display: flex; flex-direction: column; align-items: center; gap: 10px; }
  .drop-icon {
    width: 72px; height: 72px; background: #1a1a1d; border-radius: 18px;
    display: flex; align-items: center; justify-content: center; color: #3f3f46;
  }
  .drop-icon svg { width: 36px; height: 36px; }
  .drop-title { font-size: 1.1rem; font-weight: 600; color: #f4f4f5; }
  .drop-sub { font-size: 0.82rem; color: #71717a; }
  .drop-hint { font-size: 0.68rem; color: #52525b; margin-top: 4px; }

  /* Content */
  .content { display: flex; flex-direction: column; gap: 14px; height: 100%; min-height: 0; }

  .top-bar { display: flex; align-items: center; justify-content: space-between; flex-shrink: 0; }
  .top-left { display: flex; align-items: center; gap: 10px; }
  .badge {
    background: #1a1a1d; border: 1px solid #27272a; color: #a1a1aa;
    border-radius: 99px; padding: 3px 10px; font-size: 0.78rem; font-weight: 600;
  }

  /* File List */
  .file-list { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 8px; min-height: 0; }
  .file-card {
    display: flex; gap: 14px;
    background: #111113; border: 1px solid #1e1e21; border-radius: 12px;
    padding: 12px 14px; transition: border-color 0.15s; align-items: flex-start;
  }
  .file-card:hover { border-color: #27272a; }
  .file-card.done { border-color: #14532d; background: #071a0f; }

  /* Thumbnail */
  .thumb-wrap {
    width: 64px; height: 64px; flex-shrink: 0; border-radius: 8px; overflow: hidden;
    background: #1a1a1d; position: relative;
  }
  .thumb { width: 100%; height: 100%; object-fit: cover; }
  .thumb-ph {
    width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; color: #52525b;
  }
  .done-badge {
    position: absolute; bottom: 2px; right: 2px; width: 18px; height: 18px;
    background: #14532d; border: 1px solid #166534; border-radius: 50%;
    display: flex; align-items: center; justify-content: center;
    font-size: 0.65rem; font-weight: 700; color: #4ade80;
  }

  /* File body */
  .file-body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 8px; }
  .file-top { display: flex; align-items: baseline; gap: 8px; }
  .file-name { font-size: 0.875rem; font-weight: 500; color: #f4f4f5; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .file-size { font-size: 0.72rem; color: #71717a; flex-shrink: 0; }
  .loading-txt { font-size: 0.75rem; color: #71717a; font-style: italic; }

  /* Tags */
  .tags-wrap { display: flex; flex-wrap: wrap; gap: 4px; }
  .tag {
    display: inline-flex; align-items: center; gap: 4px;
    font-size: 0.68rem; padding: 2px 7px; border-radius: 5px; max-width: 260px; overflow: hidden;
  }
  .tk { font-weight: 600; opacity: 0.85; white-space: nowrap; }
  .tv { color: inherit; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .tag-red    { background: rgba(239,68,68,0.1); border: 1px solid rgba(239,68,68,0.25); color: #fca5a5; }
  .tag-blue   { background: rgba(59,130,246,0.1); border: 1px solid rgba(59,130,246,0.25); color: #93c5fd; }
  .tag-orange { background: rgba(251,146,60,0.1); border: 1px solid rgba(251,146,60,0.25); color: #fdba74; }
  .tag-gray   { background: #1a1a1d; border: 1px solid #27272a; color: #a1a1aa; }
  .tag-green  { background: rgba(74,222,128,0.1); border: 1px solid rgba(74,222,128,0.2); color: #86efac; }

  /* Action Bar */
  .action-bar {
    display: flex; align-items: center; justify-content: space-between; gap: 12px;
    background: #111113; border: 1px solid #1e1e21; border-radius: 12px;
    padding: 12px 16px; flex-shrink: 0;
  }
  .status-area { flex: 1; }
  .status { font-size: 0.82rem; font-weight: 500; }
  .status.success { color: #4ade80; }
  .status.error { color: #f87171; }
  .status.info { color: #38bdf8; }
  .action-btns { display: flex; gap: 8px; }

  /* Buttons */
  .btn {
    display: inline-flex; align-items: center; gap: 6px;
    padding: 8px 16px; border-radius: 8px;
    font-size: 0.85rem; font-weight: 600; cursor: pointer; border: none; transition: all 0.15s;
  }
  .btn.primary { background: #0ea5e9; color: white; }
  .btn.primary:hover:not(:disabled) { background: #0284c7; }
  .btn.primary:disabled { background: #1a1a1d; color: #52525b; cursor: not-allowed; }
  .btn.secondary { background: #1a1a1d; color: #e4e4e7; border: 1px solid #27272a; }
  .btn.secondary:hover { background: #27272a; }
  .btn.ghost { background: transparent; color: #71717a; border: 1px solid transparent; }
  .btn.ghost:hover { color: #e4e4e7; border-color: #27272a; }
  .btn.sm { padding: 5px 12px; font-size: 0.78rem; }

  /* Spinner */
  .spin {
    display: inline-block; width: 13px; height: 13px;
    border: 2px solid rgba(14,165,233,0.3); border-top-color: #0ea5e9;
    border-radius: 50%; animation: spin 0.7s linear infinite;
  }
  .spin.white { border-color: rgba(255,255,255,0.3); border-top-color: white; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>