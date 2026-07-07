import * as ScrollAreaPrimitive from '@radix-ui/react-scroll-area';
import type { ComponentPropsWithoutRef, Ref, UIEventHandler } from 'react';

import styles from './scroll-area.module.css';

type ScrollAreaProps = ComponentPropsWithoutRef<typeof ScrollAreaPrimitive.Root> & {
  onViewportScroll?: UIEventHandler<HTMLDivElement>;
  scrollbars?: 'both' | 'horizontal' | 'none' | 'vertical';
  viewportClassName?: string;
  viewportRef?: Ref<HTMLDivElement>;
};

export function ScrollArea({
  children,
  className,
  onViewportScroll,
  scrollbars = 'vertical',
  viewportClassName,
  viewportRef,
  ...props
}: ScrollAreaProps) {
  return (
    <ScrollAreaPrimitive.Root className={classNames(styles.root, className)} {...props}>
      <ScrollAreaPrimitive.Viewport
        className={classNames(styles.viewport, viewportClassName)}
        onScroll={onViewportScroll}
        ref={viewportRef}
      >
        {children}
      </ScrollAreaPrimitive.Viewport>
      {scrollbars === 'vertical' || scrollbars === 'both' ? (
        <ScrollBar orientation="vertical" />
      ) : null}
      {scrollbars === 'horizontal' || scrollbars === 'both' ? (
        <ScrollBar orientation="horizontal" />
      ) : null}
      {scrollbars === 'both' ? <ScrollAreaCorner /> : null}
    </ScrollAreaPrimitive.Root>
  );
}

export function ScrollBar({
  className,
  orientation = 'vertical',
  ...props
}: ComponentPropsWithoutRef<typeof ScrollAreaPrimitive.Scrollbar>) {
  return (
    <ScrollAreaPrimitive.Scrollbar
      className={classNames(styles.scrollbar, className)}
      orientation={orientation}
      {...props}
    >
      <ScrollAreaPrimitive.Thumb className={styles.thumb} />
    </ScrollAreaPrimitive.Scrollbar>
  );
}

export function ScrollAreaCorner(
  props: ComponentPropsWithoutRef<typeof ScrollAreaPrimitive.Corner>,
) {
  return <ScrollAreaPrimitive.Corner className={styles.corner} {...props} />;
}

function classNames(...values: Array<string | undefined>) {
  return values.filter(Boolean).join(' ');
}
