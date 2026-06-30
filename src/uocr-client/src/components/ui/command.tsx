import { Command as CommandPrimitive } from 'cmdk';
import type { ComponentPropsWithoutRef } from 'react';

import styles from './command.module.css';

export function Command({
  className,
  ...props
}: ComponentPropsWithoutRef<typeof CommandPrimitive>) {
  return <CommandPrimitive className={classNames(styles.root, className)} {...props} />;
}

export function CommandInput({
  className,
  ...props
}: ComponentPropsWithoutRef<typeof CommandPrimitive.Input>) {
  return <CommandPrimitive.Input className={classNames(styles.input, className)} {...props} />;
}

export function CommandList({
  className,
  ...props
}: ComponentPropsWithoutRef<typeof CommandPrimitive.List>) {
  return <CommandPrimitive.List className={classNames(styles.list, className)} {...props} />;
}

export function CommandEmpty({
  className,
  ...props
}: ComponentPropsWithoutRef<typeof CommandPrimitive.Empty>) {
  return <CommandPrimitive.Empty className={classNames(styles.empty, className)} {...props} />;
}

export function CommandGroup({
  className,
  ...props
}: ComponentPropsWithoutRef<typeof CommandPrimitive.Group>) {
  return <CommandPrimitive.Group className={classNames(styles.group, className)} {...props} />;
}

export function CommandItem({
  className,
  ...props
}: ComponentPropsWithoutRef<typeof CommandPrimitive.Item>) {
  return <CommandPrimitive.Item className={classNames(styles.item, className)} {...props} />;
}

export function CommandSeparator({
  className,
  ...props
}: ComponentPropsWithoutRef<typeof CommandPrimitive.Separator>) {
  return (
    <CommandPrimitive.Separator className={classNames(styles.separator, className)} {...props} />
  );
}

export function CommandShortcut({ className, ...props }: ComponentPropsWithoutRef<'span'>) {
  return <span className={classNames(styles.shortcut, className)} {...props} />;
}

function classNames(...values: Array<string | undefined>) {
  return values.filter(Boolean).join(' ');
}
