import { Bell } from 'lucide-react';
import { useState } from 'react';

import { markNotificationsRead, useNotificationState } from '../../stores/notificationStore';
import styles from './NotificationBell.module.css';

export function NotificationBell() {
  const [open, setOpen] = useState(false);
  const { notifications } = useNotificationState();
  const unread = notifications.filter((notification) => !notification.read).length;
  const toggleOpen = () => {
    const next = !open;
    setOpen(next);
    if (next) {
      markNotificationsRead();
    }
  };

  return (
    <div className={styles.host}>
      {open ? (
        <section className={styles.panel} aria-label="Notifications">
          <div className={styles.header}>Notifications</div>
          <div className={styles.list}>
            {notifications.length === 0 ? (
              <div className={styles.empty}>No notifications</div>
            ) : null}
            {notifications.map((notification) => (
              <article
                className={styles.item}
                data-level={notification.level}
                key={notification.id}
              >
                <div className={styles.title}>{notification.title}</div>
                {notification.message ? (
                  <div className={styles.message}>{notification.message}</div>
                ) : null}
                <time className={styles.time}>{timeLabel(notification.createdAt)}</time>
              </article>
            ))}
          </div>
        </section>
      ) : null}
      <button
        aria-expanded={open}
        aria-label="Notifications"
        className={styles.button}
        onClick={toggleOpen}
        title="Notifications"
        type="button"
      >
        <Bell size={16} />
        {unread > 0 ? <span className={styles.count}>{unread}</span> : null}
      </button>
    </div>
  );
}

function timeLabel(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}
