import { Store, useStore } from '@tanstack/react-store';

export type NotificationLevel = 'info' | 'warning' | 'error' | 'success';

export interface UiNotification {
  id: string;
  title: string;
  message?: string;
  level: NotificationLevel;
  createdAt: string;
  read: boolean;
}

interface NotificationState {
  notifications: UiNotification[];
}

const notificationStore = new Store<NotificationState>({ notifications: [] });

export function useNotificationState() {
  return useStore(notificationStore, (state) => state);
}

export function addNotification(input: {
  title: string;
  message?: string;
  level?: NotificationLevel;
}) {
  const notification: UiNotification = {
    createdAt: new Date().toISOString(),
    id: globalThis.crypto?.randomUUID?.() ?? `notification-${Date.now()}`,
    level: input.level ?? 'info',
    message: input.message,
    read: false,
    title: input.title,
  };
  notificationStore.setState((state) => ({
    notifications: [notification, ...state.notifications].slice(0, 80),
  }));
  return notification.id;
}

export function markNotificationsRead() {
  notificationStore.setState((state) => ({
    notifications: state.notifications.map((notification) => ({ ...notification, read: true })),
  }));
}
