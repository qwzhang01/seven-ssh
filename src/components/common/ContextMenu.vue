<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, nextTick } from 'vue'

export interface MenuItem {
  id: string
  label: string
  icon?: string
  danger?: boolean
  disabled?: boolean
  divider?: boolean
}

const props = defineProps<{
  items: MenuItem[]
  x: number
  y: number
}>()

const emit = defineEmits<{
  select: [id: string]
  close: []
}>()

const menuRef = ref<HTMLDivElement>()

function adjustPosition() {
  if (!menuRef.value) return
  const rect = menuRef.value.getBoundingClientRect()
  const vw = window.innerWidth
  const vh = window.innerHeight

  if (rect.right > vw) {
    menuRef.value.style.left = `${props.x - rect.width}px`
  }
  if (rect.bottom > vh) {
    menuRef.value.style.top = `${props.y - rect.height}px`
  }
}

function handleClick(item: MenuItem) {
  if (item.disabled || item.divider) return
  emit('select', item.id)
  emit('close')
}

function handleOutsideClick(e: MouseEvent) {
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
    emit('close')
  }
}

onMounted(async () => {
  await nextTick()
  adjustPosition()
  setTimeout(() => {
    document.addEventListener('mousedown', handleOutsideClick)
  }, 0)
})

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', handleOutsideClick)
})
</script>

<template>
  <Teleport to="body">
    <div
      ref="menuRef"
      class="context-menu"
      :style="{ left: x + 'px', top: y + 'px' }"
    >
      <template v-for="item in items" :key="item.id">
        <div v-if="item.divider" class="menu-divider" />
        <div
          v-else
          class="menu-item"
          :class="{ danger: item.danger, disabled: item.disabled }"
          @click="handleClick(item)"
        >
          <span v-if="item.icon" class="menu-icon">{{ item.icon }}</span>
          <span class="menu-label">{{ item.label }}</span>
        </div>
      </template>
    </div>
  </Teleport>
</template>

<style scoped>
.context-menu {
  position: fixed;
  z-index: 9999;
  min-width: 160px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  padding: 4px 0;
  animation: menu-in 0.1s ease-out;
}

@keyframes menu-in {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  font-size: 12px;
  color: var(--text-primary);
  cursor: pointer;
  transition: background var(--transition-fast);
  user-select: none;
}

.menu-item:hover:not(.disabled) {
  background: var(--bg-hover);
}

.menu-item.danger { color: var(--error); }
.menu-item.danger:hover:not(.disabled) { background: rgba(243, 139, 168, 0.1); }

.menu-item.disabled {
  color: var(--text-muted);
  cursor: default;
  opacity: 0.5;
}

.menu-icon {
  width: 16px;
  text-align: center;
  font-size: 13px;
  flex-shrink: 0;
}

.menu-label {
  flex: 1;
}

.menu-divider {
  height: 1px;
  background: var(--border-color);
  margin: 4px 8px;
}
</style>
