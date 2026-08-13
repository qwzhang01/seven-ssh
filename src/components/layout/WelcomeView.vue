<script setup lang="ts">
import AppLogo from '@/components/common/AppLogo.vue'
</script>

<template>
  <div class="welcome">
    <div class="welcome-content">
      <AppLogo :size="72" glow class="welcome-logo" />
      <h1 class="welcome-title">SevenSSH</h1>
      <p class="welcome-subtitle">Cross-platform SSH Terminal & SFTP Client</p>

      <div class="shortcuts">
        <div class="shortcut-group">
          <h3>Quick Start</h3>
          <div class="shortcut-item">
            <kbd>Double-click</kbd>
            <span>a connection in the sidebar to open a terminal</span>
          </div>
          <div class="shortcut-item">
            <kbd>⌘N</kbd>
            <span>New connection</span>
          </div>
          <div class="shortcut-item">
            <kbd>⌘K</kbd>
            <span>Quick search</span>
          </div>
        </div>
      </div>

      <p class="version">v0.1.0</p>
    </div>
  </div>
</template>

<style scoped>
.welcome {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  background: var(--bg-primary);
}

.welcome-content {
  text-align: center;
  max-width: 480px;
}

.welcome-logo {
  margin: 0 auto 20px;
}

.welcome-title {
  font-size: 36px;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.welcome-subtitle {
  font-size: 15px;
  color: var(--text-muted);
  margin-bottom: 40px;
}

.shortcuts {
  text-align: left;
  background: var(--bg-surface);
  border-radius: var(--radius-lg);
  padding: 20px 24px;
}

.shortcut-group h3 {
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 12px;
  font-weight: 500;
}

.shortcut-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 0;
  font-size: var(--font-size-md);
  color: var(--text-secondary);
}

kbd {
  display: inline-block;
  padding: 2px 8px;
  background: var(--bg-hover);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-primary);
  min-width: 100px;
  text-align: center;
}

.version {
  margin-top: 32px;
  font-size: 11px;
  color: var(--text-muted);
}
</style>
