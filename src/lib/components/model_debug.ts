import { writable } from 'svelte/store'

const LOG_KEY   = '__modelDebugLogs'
const ERROR_KEY = '__modelLoadFailed'

function loadLogs(): string[] {
  try { return JSON.parse(localStorage.getItem(LOG_KEY) ?? '[]') } catch { return [] }
}

export const debugLogs = writable<string[]>(loadLogs())

export function appendLog(msg: string) {
  debugLogs.update(l => {
    const next = [...l, `${new Date().toISOString().slice(11, 23)} ${msg}`]
    try { localStorage.setItem(LOG_KEY, JSON.stringify(next)) } catch {}
    return next
  })
}

export function clearLogs() {
  try { localStorage.removeItem(LOG_KEY) } catch {}
  debugLogs.set([])
}

export function setLoadFailed(v: boolean) {
  try {
    if (v) localStorage.setItem(ERROR_KEY, '1')
    else   localStorage.removeItem(ERROR_KEY)
  } catch {}
}

export function wasLoadFailed(): boolean {
  try { return localStorage.getItem(ERROR_KEY) === '1' } catch { return false }
}