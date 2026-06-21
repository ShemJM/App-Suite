<script lang="ts">
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';

  interface PhotoEntry {
    path: string;
    filename: string;
    timestamp: number;
    size: number;
  }

  interface MonthGroup {
    key: string;
    label: string;
    photos: PhotoEntry[];
  }

  interface BurstGroup {
    photos: PhotoEntry[];
    spanSeconds: number;
  }

  type SortKey = 'date-desc' | 'date-asc' | 'size-desc' | 'size-asc';
  type View = 'timeline' | 'bursts';

  const MONTH_NAMES = [
    'January','February','March','April','May','June',
    'July','August','September','October','November','December',
  ];

  const BURST_GAP_MS = 20_000; // 20 seconds

  // ── State ────────────────────────────────────────────────────────────────
  let folderPath = $state<string | null>(
    typeof localStorage !== 'undefined' ? localStorage.getItem('photo-vault-folder') : null
  );
  let photos = $state<PhotoEntry[]>([]);
  let scanning = $state(false);
  let scanError = $state<string | null>(null);
  let sortKey = $state<SortKey>('date-desc');
  let view = $state<View>('timeline');
  let selectedIndex = $state<number | null>(null);

  // ── Derived ──────────────────────────────────────────────────────────────
  let sortedPhotos = $derived.by((): PhotoEntry[] => {
    const s = [...photos];
    switch (sortKey) {
      case 'date-asc':  return s.sort((a, b) => a.timestamp - b.timestamp);
      case 'date-desc': return s.sort((a, b) => b.timestamp - a.timestamp);
      case 'size-asc':  return s.sort((a, b) => a.size - b.size);
      case 'size-desc': return s.sort((a, b) => b.size - a.size);
    }
  });

  let selectedPhoto = $derived(
    selectedIndex !== null ? sortedPhotos[selectedIndex] : null
  );

  let monthGroups = $derived.by((): MonthGroup[] => {
    const map = new Map<string, PhotoEntry[]>();
    for (const photo of sortedPhotos) {
      const d = new Date(photo.timestamp);
      const key = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(photo);
    }
    // Preserve display order (already sorted by sortKey)
    return Array.from(map.entries()).map(([key, group]) => {
      const [year, month] = key.split('-').map(Number);
      return { key, label: `${MONTH_NAMES[month - 1]} ${year}`, photos: group };
    });
  });

  let burstGroups = $derived.by((): BurstGroup[] => {
    // Always group by date regardless of current sort
    const byDate = [...photos].sort((a, b) => a.timestamp - b.timestamp);
    const groups: BurstGroup[] = [];
    let run: PhotoEntry[] = [];

    for (const photo of byDate) {
      if (run.length === 0) {
        run = [photo];
      } else if (photo.timestamp - run[run.length - 1].timestamp <= BURST_GAP_MS) {
        run.push(photo);
      } else {
        if (run.length >= 2) {
          groups.push({ photos: run, spanSeconds: Math.round((run[run.length - 1].timestamp - run[0].timestamp) / 1000) });
        }
        run = [photo];
      }
    }
    if (run.length >= 2) {
      groups.push({ photos: run, spanSeconds: Math.round((run[run.length - 1].timestamp - run[0].timestamp) / 1000) });
    }

    return groups.sort((a, b) => b.photos.length - a.photos.length);
  });

  // ── Actions ──────────────────────────────────────────────────────────────
  async function pickFolder() {
    const selected = await openDialog({ directory: true, multiple: false, title: 'Select Photo Folder' });
    if (!selected || Array.isArray(selected)) return;
    folderPath = selected;
    localStorage.setItem('photo-vault-folder', selected);
    await scan(selected);
  }

  async function scan(path: string) {
    scanning = true;
    scanError = null;
    photos = [];
    try {
      photos = await invoke<PhotoEntry[]>('scan_directory', { path });
    } catch (e) {
      scanError = String(e);
    } finally {
      scanning = false;
    }
  }

  $effect(() => {
    if (folderPath && photos.length === 0 && !scanning) scan(folderPath);
  });

  function selectPhoto(photo: PhotoEntry) {
    selectedIndex = sortedPhotos.indexOf(photo);
  }

  function prev() {
    if (selectedIndex !== null && selectedIndex > 0) selectedIndex -= 1;
  }

  function next() {
    if (selectedIndex !== null && selectedIndex < sortedPhotos.length - 1) selectedIndex += 1;
  }

  function closeLightbox() { selectedIndex = null; }

  async function deletePhoto() {
    if (selectedIndex === null || !selectedPhoto) return;
    const path = selectedPhoto.path;
    const idx = selectedIndex;
    photos = photos.filter(p => p.path !== path);
    selectedIndex = photos.length === 0 ? null : Math.min(idx, sortedPhotos.length - 1);
    await invoke('delete_photo', { path }).catch(console.error);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (selectedIndex === null) return;
    if (e.key === 'ArrowLeft')  { e.preventDefault(); prev(); }
    else if (e.key === 'ArrowRight') { e.preventDefault(); next(); }
    else if (e.key === 'Delete')     { e.preventDefault(); deletePhoto(); }
    else if (e.key === 'Escape')     closeLightbox();
  }

  // ── Helpers ───────────────────────────────────────────────────────────────
  const photoUrl = (path: string) => convertFileSrc(path);
  const folderName = (path: string) => path.split('/').pop() ?? path;

  function formatSize(bytes: number): string {
    if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(1)} MB`;
    if (bytes >= 1_024)     return `${Math.round(bytes / 1_024)} KB`;
    return `${bytes} B`;
  }

  function formatDate(ts: number): string {
    return new Date(ts).toLocaleDateString(undefined, { year: 'numeric', month: 'long', day: 'numeric' });
  }

  function formatTime(ts: number): string {
    return new Date(ts).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="photo-vault">
  <!-- Sidebar -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <span class="app-name">PhotoVault</span>
    </div>

    <div class="folder-info">
      {#if folderPath}
        <span class="folder-path" title={folderPath}>{folderName(folderPath)}</span>
        <button class="change-btn" onclick={pickFolder}>Change folder</button>
      {:else}
        <button class="open-folder-btn" onclick={pickFolder}>Open folder…</button>
      {/if}
    </div>

    {#if photos.length > 0}
      <div class="sort-row">
        <span class="sort-label">Sort</span>
        <select bind:value={sortKey}>
          <option value="date-desc">Date ↓</option>
          <option value="date-asc">Date ↑</option>
          <option value="size-desc">Size ↓</option>
          <option value="size-asc">Size ↑</option>
        </select>
      </div>

      <nav class="sidebar-nav">
        <button
          class="nav-item"
          class:active={view === 'timeline'}
          onclick={() => view = 'timeline'}
        >
          <span class="nav-icon">🗓</span>
          <span class="nav-label">Timeline</span>
          <span class="nav-count">{photos.length}</span>
        </button>

        <button
          class="nav-item"
          class:active={view === 'bursts'}
          onclick={() => view = 'bursts'}
        >
          <span class="nav-icon">⚡</span>
          <span class="nav-label">Burst Groups</span>
          <span class="nav-count">{burstGroups.length}</span>
        </button>

        {#if view === 'timeline'}
          <div class="month-divider"></div>
          {#each monthGroups as group}
            <a class="nav-item month-link" href="#{group.key}">
              <span class="nav-label">{group.label}</span>
              <span class="nav-count">{group.photos.length}</span>
            </a>
          {/each}
        {/if}
      </nav>
    {/if}
  </aside>

  <!-- Main content -->
  <main class="content">
    {#if scanning}
      <div class="center-state">
        <div class="spinner-large"></div>
        <p>Scanning photos…</p>
        {#if folderPath}<p class="scan-path">{folderPath}</p>{/if}
      </div>

    {:else if scanError}
      <div class="center-state">
        <p class="error-msg">⚠ {scanError}</p>
        <button class="action-btn" onclick={pickFolder}>Try another folder</button>
      </div>

    {:else if !folderPath}
      <div class="center-state">
        <span class="hero-icon">🖼</span>
        <p class="hero-title">No folder selected</p>
        <p class="hero-sub">Choose a folder to browse your photos by date</p>
        <button class="action-btn" onclick={pickFolder}>Open Folder</button>
      </div>

    {:else if photos.length === 0}
      <div class="center-state">
        <span class="hero-icon">🔍</span>
        <p class="hero-title">No photos found</p>
        <p class="hero-sub">No supported image files in this folder</p>
        <button class="action-btn" onclick={pickFolder}>Try another folder</button>
      </div>

    {:else if view === 'bursts'}
      <!-- Burst groups view -->
      {#if burstGroups.length === 0}
        <div class="center-state">
          <span class="hero-icon">✓</span>
          <p class="hero-title">No burst groups found</p>
          <p class="hero-sub">No photos taken within 20 seconds of each other</p>
        </div>
      {:else}
        <div class="timeline">
          <div class="burst-intro">
            <strong>{burstGroups.length} burst group{burstGroups.length === 1 ? '' : 's'}</strong>
            — photos taken within 20 seconds of each other. Click any photo to review it.
          </div>

          {#each burstGroups as group, gi (gi)}
            <section class="burst-section">
              <div class="month-header">
                <h2 class="month-title">
                  {formatDate(group.photos[0].timestamp)} · {formatTime(group.photos[0].timestamp)}
                </h2>
                <span class="month-count-pill">
                  {group.photos.length} photos · {group.spanSeconds}s
                </span>
              </div>
              <div class="photo-grid">
                {#each group.photos as photo (photo.path)}
                  <button
                    class="photo-thumb"
                    onclick={() => selectPhoto(photo)}
                    title={photo.filename}
                  >
                    <img src={photoUrl(photo.path)} alt={photo.filename} loading="lazy" decoding="async" />
                  </button>
                {/each}
              </div>
            </section>
          {/each}
        </div>
      {/if}

    {:else}
      <!-- Timeline view -->
      <div class="timeline">
        {#each monthGroups as group (group.key)}
          <section class="month-section" id={group.key}>
            <div class="month-header">
              <h2 class="month-title">{group.label}</h2>
              <span class="month-count-pill">{group.photos.length} photo{group.photos.length === 1 ? '' : 's'}</span>
            </div>
            <div class="photo-grid">
              {#each group.photos as photo (photo.path)}
                <button
                  class="photo-thumb"
                  onclick={() => selectPhoto(photo)}
                  title={photo.filename}
                >
                  <img src={photoUrl(photo.path)} alt={photo.filename} loading="lazy" decoding="async" />
                </button>
              {/each}
            </div>
          </section>
        {/each}
      </div>
    {/if}
  </main>
</div>

<!-- Lightbox -->
{#if selectedPhoto && selectedIndex !== null}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="lightbox-backdrop" onclick={closeLightbox}>
    <div class="lightbox" onclick={(e) => e.stopPropagation()}>
      <img src={photoUrl(selectedPhoto.path)} alt={selectedPhoto.filename} />

      <div class="lightbox-bar">
        <div class="lightbox-info">
          <span class="lightbox-name">{selectedPhoto.filename}</span>
          <span class="lightbox-meta">{formatDate(selectedPhoto.timestamp)}</span>
          <span class="lightbox-meta">{formatSize(selectedPhoto.size)}</span>
          <span class="lightbox-counter">{selectedIndex + 1} / {sortedPhotos.length}</span>
        </div>

        <div class="lightbox-actions">
          <button class="lb-btn" onclick={prev} disabled={selectedIndex === 0} title="Previous (←)">‹</button>
          <button class="lb-btn" onclick={next} disabled={selectedIndex === sortedPhotos.length - 1} title="Next (→)">›</button>
          <button class="lb-btn lb-delete" onclick={deletePhoto} title="Move to trash (Del)">🗑</button>
        </div>
      </div>

      <button class="lightbox-close" onclick={closeLightbox}>✕</button>
    </div>
  </div>
{/if}

<style>
  .photo-vault {
    display: flex;
    height: 100vh;
    background: var(--bg);
    overflow: hidden;
  }

  /* ── Sidebar ── */
  .sidebar {
    width: 200px;
    flex-shrink: 0;
    background: var(--bg-elevated);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .sidebar-header {
    padding: 1rem 1rem 0.75rem;
    border-bottom: 1px solid var(--border);
  }

  .app-name {
    font-weight: 700;
    font-size: 1rem;
    letter-spacing: -0.02em;
  }

  .folder-info {
    padding: 0.75rem;
    border-bottom: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .folder-path {
    font-size: 0.8rem;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .change-btn {
    font-size: 0.75rem;
    color: var(--accent-hover);
    text-align: left;
    padding: 0;
    transition: color 0.1s;
  }

  .change-btn:hover { color: var(--text-primary); }

  .open-folder-btn {
    width: 100%;
    padding: 0.45rem 0.75rem;
    background: var(--accent-subtle);
    border: 1px solid var(--accent);
    border-radius: var(--radius-sm);
    font-size: 0.82rem;
    color: var(--accent-hover);
    font-weight: 500;
    transition: background 0.15s;
  }

  .open-folder-btn:hover {
    background: var(--accent);
    color: #fff;
  }

  .sort-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border);
  }

  .sort-label {
    font-size: 0.75rem;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .sort-row select {
    flex: 1;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font: inherit;
    font-size: 0.78rem;
    padding: 0.2rem 0.4rem;
    cursor: pointer;
    outline: none;
  }

  .sort-row select:focus { border-color: var(--accent); }

  .sidebar-nav {
    flex: 1;
    overflow-y: auto;
    padding: 0.4rem 0;
    display: flex;
    flex-direction: column;
  }

  .month-divider {
    height: 1px;
    background: var(--border);
    margin: 0.4rem 0;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.38rem 0.75rem;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: 0.82rem;
    transition: background 0.1s, color 0.1s;
    text-align: left;
    text-decoration: none;
  }

  .nav-item:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .nav-item.active {
    background: var(--accent-subtle);
    color: var(--accent-hover);
  }

  .nav-icon { font-size: 0.85rem; width: 1.1rem; text-align: center; flex-shrink: 0; }
  .nav-label { flex: 1; }

  .nav-count {
    font-size: 0.72rem;
    color: var(--text-muted);
    background: var(--bg-hover);
    border-radius: 99px;
    padding: 0.05rem 0.4rem;
    flex-shrink: 0;
  }

  .month-link { padding-left: 1.5rem; }

  /* ── Main ── */
  .content {
    flex: 1;
    overflow-y: auto;
    min-width: 0;
  }

  .center-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 0.6rem;
    text-align: center;
    padding: 2rem;
    color: var(--text-muted);
  }

  .hero-icon    { font-size: 3rem; opacity: 0.35; }
  .hero-title   { font-size: 1rem; font-weight: 600; color: var(--text-secondary); }
  .hero-sub     { font-size: 0.85rem; }
  .error-msg    { font-size: 0.85rem; color: #f87171; max-width: 380px; }
  .scan-path    { font-size: 0.75rem; max-width: 340px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .action-btn {
    margin-top: 0.5rem;
    padding: 0.5rem 1.25rem;
    background: var(--accent);
    color: #fff;
    border-radius: var(--radius-md);
    font-size: 0.9rem;
    font-weight: 500;
    transition: background 0.15s;
  }

  .action-btn:hover { background: var(--accent-hover); }

  .spinner-large {
    width: 2rem;
    height: 2rem;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin-bottom: 0.5rem;
  }

  @keyframes spin { to { transform: rotate(360deg); } }

  /* ── Timeline / Burst shared ── */
  .timeline {
    padding: 1.5rem 1.5rem 3rem;
    display: flex;
    flex-direction: column;
    gap: 2.5rem;
  }

  .burst-intro {
    font-size: 0.83rem;
    color: var(--text-secondary);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 0.6rem 0.9rem;
  }

  .month-section, .burst-section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .month-header {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
    position: sticky;
    top: 0;
    background: var(--bg);
    padding: 0.5rem 0;
    z-index: 1;
  }

  .month-title {
    font-size: 1.05rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text-primary);
  }

  .month-count-pill {
    font-size: 0.75rem;
    color: var(--text-muted);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 99px;
    padding: 0.1rem 0.5rem;
  }

  /* ── Photo grid ── */
  .photo-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 4px;
  }

  .photo-thumb {
    aspect-ratio: 1;
    overflow: hidden;
    border-radius: var(--radius-sm);
    background: var(--bg-card);
    transition: opacity 0.15s, transform 0.15s;
    position: relative;
  }

  .photo-thumb:hover {
    opacity: 0.88;
    transform: scale(1.02);
    z-index: 1;
  }

  .photo-thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  /* ── Lightbox ── */
  .lightbox-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    backdrop-filter: blur(4px);
  }

  .lightbox {
    position: relative;
    width: min(92vw, 1100px);
    height: 84vh;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .lightbox img {
    flex: 1;
    min-height: 0;
    width: 100%;
    object-fit: contain;
    border-radius: var(--radius-md);
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.6);
  }

  .lightbox-bar {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .lightbox-info {
    display: flex;
    gap: 0.75rem;
    align-items: center;
    color: var(--text-secondary);
    font-size: 0.85rem;
    min-width: 0;
  }

  .lightbox-name {
    color: var(--text-primary);
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 240px;
  }

  .lightbox-meta {
    color: var(--text-muted);
    font-size: 0.78rem;
    flex-shrink: 0;
  }

  .lightbox-counter {
    color: var(--text-muted);
    font-size: 0.78rem;
    flex-shrink: 0;
  }

  .lightbox-actions {
    display: flex;
    gap: 0.4rem;
    flex-shrink: 0;
  }

  .lb-btn {
    width: 2.2rem;
    height: 2.2rem;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.3rem;
    border-radius: var(--radius-sm);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    color: var(--text-primary);
    transition: background 0.1s, color 0.1s, opacity 0.1s;
  }

  .lb-btn:hover:not(:disabled) { background: var(--bg-hover); }
  .lb-btn:disabled { opacity: 0.3; cursor: default; }

  .lb-delete { font-size: 1rem; border-color: transparent; }
  .lb-delete:hover:not(:disabled) {
    background: rgba(239, 68, 68, 0.15);
    border-color: rgba(239, 68, 68, 0.4);
    color: #f87171;
  }

  .lightbox-close {
    position: absolute;
    top: -2.5rem;
    right: 0;
    font-size: 1.1rem;
    color: var(--text-muted);
    width: 2rem;
    height: 2rem;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: var(--bg-elevated);
    transition: color 0.1s, background 0.1s;
  }

  .lightbox-close:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }
</style>
