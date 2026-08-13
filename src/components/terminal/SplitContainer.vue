<script setup lang="ts">
import { computed } from 'vue'

import type { SessionTab } from '@/types'
import { useSessionStore } from '@/stores/sessionStore'
import TerminalPane from './TerminalPane.vue'

const props = defineProps<{
  tab: SessionTab
}>()

const sessionStore = useSessionStore()

const layoutClass = computed(() => {
  switch (props.tab.splitDirection) {
    case 'horizontal': return 'split-horizontal'
    case 'vertical': return 'split-vertical'
    case 'grid': return 'split-grid'
    default: return 'split-none'
  }
})
</script>

<template>
  <div class="split-container" :class="layoutClass">
    <TerminalPane
      v-for="pane in tab.panes"
      :key="pane.id"
      :pane="pane"
      :is-active="sessionStore.activePaneId === pane.id"
      :sync-input="tab.syncInput"
      :sibling-panes="tab.syncInput ? tab.panes : []"
      class="pane-slot"
      @focus="sessionStore.setActivePane(pane.id)"
      @close="sessionStore.removePane(pane.id)"
    />
  </div>
</template>

<style scoped>
.split-container {
  width: 100%;
  height: 100%;
  display: flex;
  gap: 2px;
  background: var(--bg-tertiary);
}

.split-none {
  flex-direction: column;
  gap: 0;
  background: transparent;
}

.split-horizontal {
  flex-direction: row;
}

.split-vertical {
  flex-direction: column;
}

.split-grid {
  flex-wrap: wrap;
}

.pane-slot {
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  background: var(--bg-primary);
}

.split-grid .pane-slot {
  flex-basis: calc(50% - 1px);
  max-width: calc(50% - 1px);
  max-height: calc(50% - 1px);
}
</style>
