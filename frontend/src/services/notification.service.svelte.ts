import type { AppNotification } from '@types';

function createNotificationService() {
  let notifications = $state<AppNotification[]>([]);

  return {
    get notifications() {
      return notifications;
    },
    get unreadCount() {
      return notifications.filter((n) => !n.read).length;
    },
    add(type: AppNotification['type'], message: string) {
      notifications = [
        {
          id: crypto.randomUUID(),
          type,
          message,
          timestamp: new Date().toISOString(),
          read: false,
        },
        ...notifications,
      ].slice(0, 50);
    },
    markAllRead() {
      notifications = notifications.map((n) => ({ ...n, read: true }));
    },
    clear() {
      notifications = [];
    },
    _reset() {
      notifications = [];
    },
  };
}

export const notificationService = createNotificationService();
