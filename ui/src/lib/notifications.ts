export type NotificationLevel = 'debug' | 'info' | 'warn' | 'error'
export type NotificationPosition = 'top_left' | 'top_right' | 'bottom_left' | 'bottom_right'

export type Notification = {
  id: number
  level: NotificationLevel
  message: string
}

type NotificationListener = (notifications: Notification[]) => void
type NotificationPositionListener = (position: NotificationPosition) => void

const listeners = new Set<NotificationListener>()
const positionListeners = new Set<NotificationPositionListener>()
let notifications: Notification[] = []
let nextId = 1
let defaultPosition: NotificationPosition = 'top_right'

export function notify(message: string, level: NotificationLevel = 'info', timeout = 5000): number {
  const id = nextId++
  notifications = [...notifications, { id, level, message }]
  publish()

  if (timeout >= 0) {
    window.setTimeout(() => dismiss(id), timeout)
  }
  return id
}

export function debug(message: string, timeout = 5000): number {
  return notify(message, 'debug', timeout)
}

export function info(message: string, timeout = 5000): number {
  return notify(message, 'info', timeout)
}

export function warn(message: string, timeout = 5000): number {
  return notify(message, 'warn', timeout)
}

export function error(message: string, timeout = 5000): number {
  return notify(message, 'error', timeout)
}

export function dismiss(id: number): void {
  const next = notifications.filter((notification) => notification.id !== id)
  if (next.length === notifications.length) return
  notifications = next
  publish()
}

export function getNotifications(): Notification[] {
  return notifications
}

export function subscribeNotifications(listener: NotificationListener): () => void {
  listeners.add(listener)
  listener(notifications)
  return () => listeners.delete(listener)
}

export function setNotificationPosition(position: string): void {
  if (!isNotificationPosition(position) || defaultPosition === position) return
  defaultPosition = position
  for (const listener of positionListeners) listener(defaultPosition)
}

export function getNotificationPosition(): NotificationPosition {
  return defaultPosition
}

export function subscribeNotificationPosition(listener: NotificationPositionListener): () => void {
  positionListeners.add(listener)
  listener(defaultPosition)
  return () => positionListeners.delete(listener)
}

function isNotificationPosition(value: string): value is NotificationPosition {
  return value === 'top_left'
    || value === 'top_right'
    || value === 'bottom_left'
    || value === 'bottom_right'
}

function publish(): void {
  for (const listener of listeners) listener(notifications)
}
