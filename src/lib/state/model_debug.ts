const ERROR_KEY = '__modelLoadFailed'

export function setLoadFailed(v: boolean) {
    try {
        if (v) localStorage.setItem(ERROR_KEY, '1')
        else   localStorage.removeItem(ERROR_KEY)
    } catch {}
}

export function wasLoadFailed(): boolean {
    try { return localStorage.getItem(ERROR_KEY) === '1' } catch { return false }
}