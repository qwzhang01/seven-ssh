<script setup lang="ts">
const shortcuts = [
  { action: 'Split Horizontal', keys: '⌘ ⇧ H', context: 'Terminal' },
  { action: 'Split Vertical', keys: '⌘ ⌥ ⇧ V', context: 'Terminal' },
  { action: 'Command Palette', keys: '⌘ ⇧ P', context: 'Global' },
  { action: 'Terminal Search', keys: '⌘ F', context: 'Terminal (focused)' },
  { action: 'Settings', keys: '⌘ ,', context: 'Global' },
  { action: 'Lock App', keys: '⌘ ⇧ L', context: 'Global' },
  { action: 'SFTP Browser', keys: '—', context: 'Context menu' },
  { action: 'AI Chat', keys: '⌘ ⇧ A', context: 'Global' },
  { action: 'Inline Assist', keys: '⌘ I', context: 'Terminal (focused)' },
  { action: 'Close Tab', keys: '⌘ W', context: 'Global' },
  { action: 'Next Tab', keys: '⌘ ⇧ ]', context: 'Global' },
  { action: 'Previous Tab', keys: '⌘ ⇧ [', context: 'Global' },
  { action: 'New Connection', keys: '⌘ N', context: 'Sidebar' },
  { action: 'Toggle Sidebar', keys: '⌘ B', context: 'Global' },
]
</script>

<template>
  <div class="shortcut-settings">
    <p class="shortcut-note">
      Keyboard shortcuts reference. Custom keybindings will be available in a future release.
    </p>
    <div class="shortcut-table-wrap">
      <table class="shortcut-table">
        <thead>
          <tr>
            <th>Action</th>
            <th>Shortcut</th>
            <th>Context</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(item, idx) in shortcuts" :key="idx">
            <td class="col-action">{{ item.action }}</td>
            <td class="col-keys">
              <kbd v-if="item.keys !== '—'">{{ item.keys }}</kbd>
              <span v-else class="no-key">{{ item.keys }}</span>
            </td>
            <td class="col-context">{{ item.context }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.shortcut-settings {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.shortcut-note {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.4;
}

.shortcut-table-wrap {
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.shortcut-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.shortcut-table th {
  text-align: left;
  padding: 8px 12px;
  background: var(--bg-surface);
  color: var(--text-secondary);
  font-weight: 500;
  font-size: 12px;
  border-bottom: 1px solid var(--border-color);
}

.shortcut-table td {
  padding: 8px 12px;
  border-bottom: 1px solid color-mix(in srgb, var(--border-color) 50%, transparent);
  color: var(--text-primary);
}

.shortcut-table tbody tr:last-child td {
  border-bottom: none;
}

.shortcut-table tbody tr:hover {
  background: var(--bg-hover);
}

.col-action {
  font-weight: 500;
}

.col-keys kbd {
  display: inline-block;
  padding: 2px 8px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--text-primary);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.15);
}

.no-key {
  color: var(--text-muted);
  font-size: 12px;
}

.col-context {
  color: var(--text-muted);
  font-size: 12px;
}
</style>
