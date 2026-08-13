import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'

import App from './App.vue'
import i18n from './i18n'
import './styles/global.css'

// --- Error Collection ---
// TODO: Replace with Sentry for production error tracking:
//   import * as Sentry from '@sentry/vue'
//   Sentry.init({ app, dsn: 'YOUR_SENTRY_DSN', tracesSampleRate: 0.2 })

function reportError(error: { message: string; source?: string; line?: number; col?: number; stack?: string }) {
  console.error('[SevenSSH Error]', JSON.stringify({
    timestamp: new Date().toISOString(),
    ...error,
  }))
}

window.onerror = (message, source, lineno, colno, error) => {
  reportError({
    message: String(message),
    source: source ?? undefined,
    line: lineno ?? undefined,
    col: colno ?? undefined,
    stack: error?.stack,
  })
}

window.onunhandledrejection = (event: PromiseRejectionEvent) => {
  const reason = event.reason
  reportError({
    message: reason instanceof Error ? reason.message : String(reason),
    stack: reason instanceof Error ? reason.stack : undefined,
    source: 'unhandledrejection',
  })
}

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'main',
      component: () => import('@/views/MainView.vue'),
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue'),
    },
  ],
})

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.use(i18n)

app.config.errorHandler = (err, _instance, info) => {
  console.error('[Global Error]', err, info)
  const msg = err instanceof Error ? err.message : String(err)
  showErrorToast(msg)
}

function showErrorToast(message: string) {
  const toast = document.createElement('div')
  toast.className = 'global-error-toast'
  toast.textContent = message
  document.body.appendChild(toast)
  requestAnimationFrame(() => toast.classList.add('visible'))
  setTimeout(() => {
    toast.classList.remove('visible')
    setTimeout(() => toast.remove(), 300)
  }, 4000)
}

app.mount('#app')
