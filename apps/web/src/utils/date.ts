export function parseDate(dateVal?: string | number | null): Date | null {
  if (dateVal === undefined || dateVal === null || dateVal === '') return null
  if (typeof dateVal === 'number') {
    const d = new Date(dateVal)
    return Number.isNaN(d.getTime()) ? null : d
  }
  const str = String(dateVal).trim()
  if (!str) return null
  if (/^\d+$/.test(str)) {
    const num = Number(str)
    const ms = str.length === 10 ? num * 1000 : num
    const d = new Date(ms)
    return Number.isNaN(d.getTime()) ? null : d
  }
  const d = new Date(str)
  return Number.isNaN(d.getTime()) ? null : d
}

export function formatDateTime(
  dateVal?: string | number | null,
  fallback = '-',
): string {
  const d = parseDate(dateVal)
  if (!d) return fallback
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

export function formatLocaleDate(
  dateVal?: string | number | null,
  fallback = 'Unknown',
): string {
  const d = parseDate(dateVal)
  if (!d) return fallback
  return d.toLocaleString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}
