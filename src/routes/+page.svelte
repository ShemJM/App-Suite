<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  interface App {
    label: string;
    title: string;
    route: string;
    description: string;
    icon: string;
    width: number;
    height: number;
    color: string;
  }

  const apps: App[] = [
    {
      label: 'photo-vault',
      title: 'PhotoVault',
      route: 'photo-vault',
      description: 'Organise, browse, and manage your photo library',
      icon: '🖼',
      width: 1280,
      height: 820,
      color: '#8b5cf6',
    },
  ];

  let launching: string | null = $state(null);

  async function launch(app: App) {
    if (launching) return;
    launching = app.label;
    try {
      await invoke('open_app_window', {
        label: app.label,
        title: app.title,
        route: app.route,
        width: app.width,
        height: app.height,
      });
    } finally {
      launching = null;
    }
  }
</script>

<div class="launcher">
  <header>
    <h1>App Suite</h1>
    <p>Your personal productivity tools</p>
  </header>

  <section class="grid">
    {#each apps as app}
      <button
        class="card"
        class:loading={launching === app.label}
        onclick={() => launch(app)}
        style="--card-color: {app.color}"
        disabled={launching !== null}
      >
        <span class="icon">{app.icon}</span>
        <div class="info">
          <span class="name">{app.title}</span>
          <span class="desc">{app.description}</span>
        </div>
        {#if launching === app.label}
          <span class="spinner" aria-hidden="true"></span>
        {:else}
          <span class="arrow">›</span>
        {/if}
      </button>
    {/each}

    <div class="card coming-soon">
      <span class="icon">＋</span>
      <div class="info">
        <span class="name">More coming soon</span>
        <span class="desc">Additional tools are in development</span>
      </div>
    </div>
  </section>
</div>

<style>
  .launcher {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: 2rem;
    gap: 2.5rem;
  }

  header {
    text-align: center;
  }

  header h1 {
    font-size: 2rem;
    font-weight: 700;
    letter-spacing: -0.03em;
    color: var(--text-primary);
  }

  header p {
    margin-top: 0.4rem;
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .grid {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    width: 100%;
    max-width: 520px;
  }

  .card {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem 1.25rem;
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    transition: background 0.15s, border-color 0.15s, transform 0.1s;
    text-align: left;
  }

  .card:not(.coming-soon):hover {
    background: var(--bg-hover);
    border-color: var(--card-color, var(--accent));
    transform: translateY(-1px);
  }

  .card:not(.coming-soon):active {
    transform: translateY(0);
  }

  .card.loading {
    opacity: 0.7;
  }

  .coming-soon {
    opacity: 0.4;
    cursor: default;
  }

  .icon {
    font-size: 1.75rem;
    width: 2.5rem;
    text-align: center;
    flex-shrink: 0;
  }

  .info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .name {
    font-weight: 600;
    font-size: 0.95rem;
    color: var(--text-primary);
  }

  .desc {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .arrow {
    font-size: 1.4rem;
    color: var(--text-muted);
    transition: color 0.15s;
  }

  .card:hover .arrow {
    color: var(--card-color, var(--accent));
  }

  .spinner {
    width: 1rem;
    height: 1rem;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    flex-shrink: 0;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
