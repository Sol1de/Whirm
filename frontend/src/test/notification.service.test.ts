import { describe, it, expect, beforeEach } from 'vitest';

describe('notificationService', () => {
  let notificationService: Awaited<typeof import('../services/notification.service.svelte')>['notificationService'];

  beforeEach(async () => {
    const mod = await import('../services/notification.service.svelte');
    notificationService = mod.notificationService;
    notificationService._reset();
  });

  it('starts empty', () => {
    expect(notificationService.notifications).toHaveLength(0);
    expect(notificationService.unreadCount).toBe(0);
  });

  it('adds a notification', () => {
    notificationService.add('success', 'Connected to proxy');
    expect(notificationService.notifications).toHaveLength(1);
    const n = notificationService.notifications[0];
    expect(n.type).toBe('success');
    expect(n.message).toBe('Connected to proxy');
    expect(n.read).toBe(false);
    expect(n.id).toBeTruthy();
    expect(n.timestamp).toBeTruthy();
  });

  it('prepends notifications (newest first)', () => {
    notificationService.add('info', 'first');
    notificationService.add('success', 'second');
    expect(notificationService.notifications[0].message).toBe('second');
    expect(notificationService.notifications[1].message).toBe('first');
  });

  it('tracks unread count correctly', () => {
    notificationService.add('info', 'a');
    notificationService.add('error', 'b');
    expect(notificationService.unreadCount).toBe(2);
  });

  it('markAllRead sets all to read', () => {
    notificationService.add('info', 'a');
    notificationService.add('success', 'b');
    notificationService.markAllRead();
    expect(notificationService.unreadCount).toBe(0);
    expect(notificationService.notifications.every((n) => n.read)).toBe(true);
  });

  it('clear empties the list', () => {
    notificationService.add('info', 'a');
    notificationService.clear();
    expect(notificationService.notifications).toHaveLength(0);
  });

  it('caps at 50 notifications', () => {
    for (let i = 0; i < 55; i++) {
      notificationService.add('info', `Notification ${i}`);
    }
    expect(notificationService.notifications).toHaveLength(50);
    expect(notificationService.notifications[0].message).toBe('Notification 54');
  });
});
