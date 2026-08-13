import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

import type { ConnectionInfo, GroupInfo, CreateConnectionRequest, UpdateConnectionRequest, CreateGroupRequest } from '@/types'
import * as sshService from '@/services/ssh'

export const useConnectionStore = defineStore('connection', () => {
  const connections = ref<ConnectionInfo[]>([])
  const groups = ref<GroupInfo[]>([])
  const loading = ref(false)

  const favorites = computed(() =>
    connections.value.filter((c) => c.is_favorite)
  )

  const ungrouped = computed(() =>
    connections.value.filter((c) => !c.group_id)
  )

  function getConnectionsByGroup(groupId: string): ConnectionInfo[] {
    return connections.value.filter((c) => c.group_id === groupId)
  }

  function getChildGroups(parentId: string | null): GroupInfo[] {
    return groups.value.filter((g) =>
      parentId ? g.parent_id === parentId : !g.parent_id
    )
  }

  async function fetchConnections() {
    loading.value = true
    try {
      connections.value = await sshService.listConnections()
    } finally {
      loading.value = false
    }
  }

  async function fetchGroups() {
    groups.value = await sshService.listGroups()
  }

  async function addConnection(request: CreateConnectionRequest): Promise<ConnectionInfo> {
    const conn = await sshService.createConnection(request)
    connections.value.push(conn)
    return conn
  }

  async function removeConnection(id: string) {
    await sshService.deleteConnection(id)
    connections.value = connections.value.filter((c) => c.id !== id)
  }

  async function addGroup(request: CreateGroupRequest): Promise<GroupInfo> {
    const group = await sshService.createGroup(request)
    groups.value.push(group)
    return group
  }

  async function removeGroup(id: string) {
    await sshService.deleteGroup(id)
    groups.value = groups.value.filter((g) => g.id !== id)
  }

  async function updateConnection(request: UpdateConnectionRequest): Promise<ConnectionInfo> {
    const updated = await sshService.updateConnection(request)
    const idx = connections.value.findIndex((c) => c.id === request.id)
    if (idx !== -1) connections.value[idx] = updated
    return updated
  }

  async function toggleFavorite(id: string) {
    const conn = connections.value.find((c) => c.id === id)
    if (!conn) return
    const updated = await sshService.updateConnection({
      id,
      is_favorite: !conn.is_favorite,
    })
    const idx = connections.value.findIndex((c) => c.id === id)
    if (idx !== -1) connections.value[idx] = updated
  }

  async function updateGroupOrder(id: string, sortOrder: number) {
    await sshService.updateGroup(id, undefined, undefined, undefined, sortOrder)
    const idx = groups.value.findIndex((g) => g.id === id)
    if (idx !== -1) groups.value[idx].sort_order = sortOrder
  }

  async function init() {
    await Promise.all([fetchConnections(), fetchGroups()])
  }

  return {
    connections,
    groups,
    loading,
    favorites,
    ungrouped,
    getConnectionsByGroup,
    getChildGroups,
    fetchConnections,
    fetchGroups,
    addConnection,
    updateConnection,
    removeConnection,
    addGroup,
    removeGroup,
    toggleFavorite,
    updateGroupOrder,
    init,
  }
})
