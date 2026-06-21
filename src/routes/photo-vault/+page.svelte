<script lang="ts">
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';

  interface PhotoEntry {
    path: string;
    filename: string;
    timestamp: number;
  }

  interface MonthGroup {
    key: string;       // "2024-06"
    label: string;     // "June 2024"
    photos: PhotoEntry[];
  }

  const MONTH_NAMES = [
    'January', 'February', 'March', 'April', 'May', 'June',
    'July', 'August', 'September', 'October', 'November', 'December',
  ];

  let folderPath = $state<string | null>(
    typeof localStorage !== 'undefined' ? localStorage.getItem('photo-vault-folder') : null
  );
  let photos = $state<PhotoEntry[]>([]);
  let scanning = $state(false);
  let scanError = $state<string | null>(null);
  let selectedIndex = $state<number | null>(null);
  let selectedPhoto = $derived(selectedIndex !== null ? photos[selectedIndex] : null);

  let monthGroups = $derived.by((): MonthGroup[] => {
    if (!photos.length) return [];

    const map = new Map<string, PhotoEntry[]>();
    for (const photo of photos) {
      const d = new Date(photo.timestamp);
      const key = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(photo);
    }

    return Array.from(map.entries())
      .sort(([a], [b]) => b.localeCompare(a))
      .map(([key, group]) => {
        const [year, month] = key.split('-').map(Number);
        return {
          key,
          label: `${MONTH_NAMES[month - 1]} ${year}`,
          photos: group,
        };
      });
  });

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
    if (folderPath && photos.length === 0 && !scanning) {
      scan(folderPath);
    }
  });

  function photoUrl(path: string) {
    return convertFileSrc(path);
  }

  function folderName(path: string) {
    return path.split('/').pop() ?? path;
  }

  function selectPhoto(photo: PhotoEntry) {
    selectedIndex = photos.indexOf(photo);
  }

  function prev() {
    if (selectedIndex !== null && selectedIndex > 0) selectedIndex -= 1;
  }

  function next() {
    if (selectedIndex !== null && selectedIndex < photos.length - 1) selectedIndex += 1;
  }

  function closeLightbox() {
    selectedIndex = null;
  }

  async function deletePhoto() {
    if (selectedIndex === null || !selectedPhoto) return;
    const path = selectedPhoto.path;
    const idx = selectedIndex;

    // Remove from list first for instant feedback
    photos = photos.filter((_, i) => i !== idx);

    if (photos.length === 0) {
      selectedIndex = null;
    } else {
      selectedIndex = Math.min(idx, photos.length - 1);
    }

    await invoke('delete_photo', { path }).catch(console.error);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (selectedIndex === null) return;
    if (e.key === 'ArrowLeft') { e.preventDefault(); prev(); }
    else if (e.key === 'ArrowRight') { e.preventDefault(); next(); }
    else if (e.key === 'Escape') closeLightbox();
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

    {#if monthGroups.length > 0}
      <nav class="month-nav">
        {#each monthGroups as group}
          <a class="month-link" href="#{group.key}">
            <span class="month-name">{group.label}</span>
            <span class="month-count">{group.photos.length}</span>
          </a>
        {/each}
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

    {:else}
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
                  <img
                    src={photoUrl(photo.path)}
                    alt={photo.filename}
                    loading="lazy"
                    decoding="async"
                  />
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
          <span class="lightbox-date">
            {new Date(selectedPhoto.timestamp).toLocaleDateString(undefined, {
              year: 'numeric', month: 'long', day: 'numeric'
            })}
          </span>
          <span class="lightbox-counter">{selectedIndex + 1} / {photos.length}</span>
        </div>

        <div class="lightbox-actions">
          <button class="lb-btn" onclick={prev} disabled={selectedIndex === 0} title="Previous (←)">‹</button>
          <button class="lb-btn" onclick={next} disabled={selectedIndex === photos.length - 1} title="Next (→)">›</button>
          <button class="lb-btn lb-delete" onclick={deletePhoto} title="Move to trash">🗑</button>
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

  .change-btn:hover {
    color: var(--text-primary);
  }

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

  .month-nav {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem 0;
  }

  .month-link {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.35rem 0.75rem;
    font-size: 0.82rem;
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    transition: background 0.1s, color 0.1s;
  }

  .month-link:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .month-name { flex: 1; }

  .month-count {
    font-size: 0.72rem;
    color: var(--text-muted);
    background: var(--bg-hover);
    border-radius: 99px;
    padding: 0.05rem 0.4rem;
  }

  /* ── Main content ── */
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

  .hero-icon {
    font-size: 3rem;
    opacity: 0.35;
  }

  .hero-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .hero-sub {
    font-size: 0.85rem;
  }

  .error-msg {
    font-size: 0.85rem;
    color: #f87171;
    max-width: 380px;
  }

  .scan-path {
    font-size: 0.75rem;
    max-width: 340px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

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

  .action-btn:hover {
    background: var(--accent-hover);
  }

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

  /* ── Timeline ── */
  .timeline {
    padding: 1.5rem 1.5rem 3rem;
    display: flex;
    flex-direction: column;
    gap: 2.5rem;
  }

  .month-section {
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
    font-size: 1.1rem;
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
    max-width: min(90vw, 1200px);
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
  }

  .lightbox img {
    max-width: 100%;
    max-height: calc(90vh - 60px);
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
    max-width: 280px;
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

  .lb-btn:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  .lb-btn:disabled {
    opacity: 0.3;
    cursor: default;
  }

  .lb-delete {
    font-size: 1rem;
    border-color: transparent;
  }

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
