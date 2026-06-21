<script lang="ts">
  type ViewMode = 'grid' | 'list';
  type SidebarSection = 'library' | 'albums' | 'smart';

  let viewMode = $state<ViewMode>('grid');
  let selectedFolder = $state<string | null>('All Photos');
  let searchQuery = $state('');

  const sidebarItems: Record<SidebarSection, { label: string; icon: string; count?: number }[]> = {
    library: [
      { label: 'All Photos', icon: '🖼', count: 0 },
      { label: 'Recents', icon: '🕐', count: 0 },
      { label: 'Imports', icon: '📥', count: 0 },
    ],
    albums: [],
    smart: [
      { label: 'Favourites', icon: '♥', count: 0 },
      { label: 'Videos', icon: '🎬', count: 0 },
      { label: 'Screenshots', icon: '📸', count: 0 },
    ],
  };

  function selectFolder(label: string) {
    selectedFolder = label;
  }
</script>

<div class="photo-vault">
  <!-- Sidebar -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <span class="app-name">PhotoVault</span>
    </div>

    <nav class="sidebar-nav">
      <div class="nav-section">
        <span class="section-label">Library</span>
        {#each sidebarItems.library as item}
          <button
            class="nav-item"
            class:active={selectedFolder === item.label}
            onclick={() => selectFolder(item.label)}
          >
            <span class="nav-icon">{item.icon}</span>
            <span class="nav-label">{item.label}</span>
            {#if item.count !== undefined}
              <span class="nav-count">{item.count}</span>
            {/if}
          </button>
        {/each}
      </div>

      <div class="nav-section">
        <span class="section-label">Smart Albums</span>
        {#each sidebarItems.smart as item}
          <button
            class="nav-item"
            class:active={selectedFolder === item.label}
            onclick={() => selectFolder(item.label)}
          >
            <span class="nav-icon">{item.icon}</span>
            <span class="nav-label">{item.label}</span>
          </button>
        {/each}
      </div>

      <div class="nav-section">
        <div class="section-label-row">
          <span class="section-label">Albums</span>
          <button class="add-btn" title="New album">＋</button>
        </div>
        {#if sidebarItems.albums.length === 0}
          <span class="empty-hint">No albums yet</span>
        {/if}
      </div>
    </nav>
  </aside>

  <!-- Main content -->
  <main class="content">
    <div class="toolbar">
      <h2 class="folder-title">{selectedFolder ?? 'Library'}</h2>

      <div class="toolbar-right">
        <div class="search-box">
          <span class="search-icon">⌕</span>
          <input
            type="search"
            placeholder="Search photos…"
            bind:value={searchQuery}
          />
        </div>

        <div class="view-toggle">
          <button
            class:active={viewMode === 'grid'}
            onclick={() => viewMode = 'grid'}
            title="Grid view"
          >⊞</button>
          <button
            class:active={viewMode === 'list'}
            onclick={() => viewMode = 'list'}
            title="List view"
          >☰</button>
        </div>

        <button class="import-btn">Import</button>
      </div>
    </div>

    <div class="photo-area">
      <div class="empty-state">
        <span class="empty-icon">🖼</span>
        <p class="empty-title">No photos here yet</p>
        <p class="empty-sub">Import a folder to get started</p>
        <button class="import-btn-large">Import Photos</button>
      </div>
    </div>
  </main>
</div>

<style>
  .photo-vault {
    display: flex;
    height: 100vh;
    background: var(--bg);
    overflow: hidden;
  }

  /* Sidebar */
  .sidebar {
    width: 220px;
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
    color: var(--text-primary);
  }

  .sidebar-nav {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem 0;
  }

  .nav-section {
    padding: 0.5rem 0;
    border-bottom: 1px solid var(--border);
  }

  .nav-section:last-child {
    border-bottom: none;
  }

  .section-label {
    display: block;
    padding: 0.25rem 1rem;
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
  }

  .section-label-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-right: 0.75rem;
  }

  .add-btn {
    font-size: 1rem;
    color: var(--text-muted);
    line-height: 1;
    padding: 0.1rem 0.25rem;
    border-radius: var(--radius-sm);
    transition: color 0.1s, background 0.1s;
  }

  .add-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.4rem 0.75rem 0.4rem 1rem;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: 0.85rem;
    transition: background 0.1s, color 0.1s;
    text-align: left;
  }

  .nav-item:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .nav-item.active {
    background: var(--accent-subtle);
    color: var(--accent-hover);
  }

  .nav-icon {
    font-size: 0.9rem;
    width: 1.2rem;
    text-align: center;
    flex-shrink: 0;
  }

  .nav-label {
    flex: 1;
  }

  .nav-count {
    font-size: 0.75rem;
    color: var(--text-muted);
    background: var(--bg-hover);
    border-radius: 99px;
    padding: 0.05rem 0.4rem;
  }

  .empty-hint {
    display: block;
    padding: 0.25rem 1rem;
    font-size: 0.8rem;
    color: var(--text-muted);
    font-style: italic;
  }

  /* Main content */
  .content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1.25rem;
    border-bottom: 1px solid var(--border);
    gap: 1rem;
    background: var(--bg-elevated);
  }

  .folder-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
    flex-shrink: 0;
  }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 0.35rem 0.75rem;
  }

  .search-icon {
    color: var(--text-muted);
    font-size: 1rem;
  }

  .search-box input {
    border: none;
    background: none;
    outline: none;
    color: var(--text-primary);
    font: inherit;
    font-size: 0.85rem;
    width: 180px;
  }

  .search-box input::placeholder {
    color: var(--text-muted);
  }

  .view-toggle {
    display: flex;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .view-toggle button {
    padding: 0.3rem 0.6rem;
    font-size: 0.95rem;
    color: var(--text-muted);
    background: var(--bg-card);
    transition: background 0.1s, color 0.1s;
  }

  .view-toggle button.active {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .import-btn {
    padding: 0.35rem 0.9rem;
    background: var(--accent);
    color: #fff;
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    font-weight: 500;
    transition: background 0.15s;
  }

  .import-btn:hover {
    background: var(--accent-hover);
  }

  /* Photo area */
  .photo-area {
    flex: 1;
    overflow-y: auto;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    text-align: center;
    color: var(--text-muted);
  }

  .empty-icon {
    font-size: 3rem;
    opacity: 0.4;
  }

  .empty-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-secondary);
    margin-top: 0.25rem;
  }

  .empty-sub {
    font-size: 0.85rem;
  }

  .import-btn-large {
    margin-top: 0.75rem;
    padding: 0.5rem 1.25rem;
    background: var(--accent);
    color: #fff;
    border-radius: var(--radius-md);
    font-size: 0.9rem;
    font-weight: 500;
    transition: background 0.15s;
  }

  .import-btn-large:hover {
    background: var(--accent-hover);
  }
</style>
