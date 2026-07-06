import { ChevronDown, ChevronRight } from 'lucide-react';
import type { CSSProperties } from 'react';
import { useEffect, useRef } from 'react';

import type { TreeGridNode, TreeNode } from '../types';
import styles from './Tree.module.css';

export function TreeView({
  className,
  expandedIds,
  nodes,
  onToggle,
}: {
  nodes: TreeNode[];
  expandedIds: Set<string>;
  onToggle: (id: string) => void;
  className?: string;
}) {
  return (
    <div className={classNames(styles.treeView, className)}>
      {nodes.map((node) => (
        <TreeViewRow
          expandedIds={expandedIds}
          key={node.id}
          level={0}
          node={node}
          onToggle={onToggle}
        />
      ))}
    </div>
  );
}

export function TreeGrid({
  className,
  expandedIds,
  nodes,
  onToggle,
}: {
  nodes: TreeGridNode[];
  expandedIds: Set<string>;
  onToggle: (id: string) => void;
  className?: string;
}) {
  return (
    <div className={classNames(styles.treeGrid, className)}>
      {nodes.map((node) => (
        <TreeGridRow
          expandedIds={expandedIds}
          key={node.id}
          level={0}
          node={node}
          onToggle={onToggle}
        />
      ))}
    </div>
  );
}

function TreeViewRow({
  expandedIds,
  level,
  node,
  onToggle,
}: {
  node: TreeNode;
  level: number;
  expandedIds: Set<string>;
  onToggle: (id: string) => void;
}) {
  const childCount = node.children?.length ?? 0;
  const hasChildren = node.hasChildren ?? childCount > 0;
  const expanded = expandedIds.has(node.id);
  const onRowToggle = () => {
    if (!expanded && childCount === 0) {
      node.onExpand?.();
    }
    onToggle(node.id);
  };
  return (
    <>
      <div
        className={classNames(
          styles.treeRow,
          node.checked !== undefined ? styles.treeRowWithCheckbox : undefined,
          node.selected ? styles.active : undefined,
        )}
        id={node.id}
        style={treeLevelStyle(level)}
      >
        <TreeTwisty
          expanded={expanded}
          hasChildren={hasChildren}
          label={node.label}
          onToggle={onRowToggle}
        />
        {node.checked !== undefined && (
          <TreeCheckbox checked={node.checked} onChange={node.onCheckedChange} />
        )}
        <TreeLabelButton icon={node.icon} label={node.label} onSelect={node.onSelect} />
        {node.badge && <small>{node.badge}</small>}
      </div>
      {hasChildren && expanded && (
        <div>
          {node.children?.map((child) => (
            <TreeViewRow
              expandedIds={expandedIds}
              key={child.id}
              level={level + 1}
              node={child}
              onToggle={onToggle}
            />
          ))}
        </div>
      )}
    </>
  );
}

function TreeGridRow({
  expandedIds,
  level,
  node,
  onToggle,
}: {
  node: TreeGridNode;
  level: number;
  expandedIds: Set<string>;
  onToggle: (id: string) => void;
}) {
  const childCount = node.children?.length ?? 0;
  const hasChildren = node.hasChildren ?? childCount > 0;
  const expanded = expandedIds.has(node.id);
  const onRowToggle = () => {
    if (!expanded && childCount === 0) {
      node.onExpand?.();
    }
    onToggle(node.id);
  };
  return (
    <>
      <div
        className={classNames(styles.treeGridRow, node.selected ? styles.active : undefined)}
        id={node.id}
        style={treeLevelStyle(level)}
      >
        <div className={styles.treeGridName}>
          <TreeTwisty
            expanded={expanded}
            hasChildren={hasChildren}
            label={node.label}
            onToggle={onRowToggle}
          />
          <TreeLabelButton icon={node.icon} label={node.label} onSelect={node.onSelect} />
        </div>
        <span className={styles.treeGridBadge}>{node.badge}</span>
        <span className={styles.treeGridSwitchCell}>
          {node.actions}
          {node.checked !== undefined && (
            <TreeCheckbox checked={node.checked} onChange={node.onCheckedChange} />
          )}
        </span>
      </div>
      {hasChildren && expanded && (
        <div>
          {node.children?.map((child) => (
            <TreeGridRow
              expandedIds={expandedIds}
              key={child.id}
              level={level + 1}
              node={child}
              onToggle={onToggle}
            />
          ))}
        </div>
      )}
    </>
  );
}

function TreeTwisty({
  expanded,
  hasChildren,
  label,
  onToggle,
}: {
  expanded: boolean;
  hasChildren: boolean;
  label: string;
  onToggle: () => void;
}) {
  return (
    <button
      aria-label={expanded ? `Collapse ${label}` : `Expand ${label}`}
      className={styles.treeTwisty}
      disabled={!hasChildren}
      onClick={onToggle}
      type="button"
    >
      {hasChildren ? expanded ? <ChevronDown size={13} /> : <ChevronRight size={13} /> : null}
    </button>
  );
}

function TreeLabelButton({ icon, label, onSelect }: Pick<TreeNode, 'icon' | 'label' | 'onSelect'>) {
  return (
    <button className={styles.treeLabelButton} onClick={onSelect} type="button">
      {icon}
      <span>{label}</span>
    </button>
  );
}

function TreeCheckbox({
  checked,
  onChange,
}: {
  checked: boolean | 'indeterminate';
  onChange?: (checked: boolean) => void;
}) {
  const inputRef = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.indeterminate = checked === 'indeterminate';
    }
  }, [checked]);

  return (
    <input
      ref={inputRef}
      className={styles.treeCheckbox}
      checked={checked === true}
      onChange={(event) => onChange?.(event.currentTarget.checked)}
      type="checkbox"
    />
  );
}

function treeLevelStyle(level: number): CSSProperties {
  return { '--tree-level': String(level) } as CSSProperties;
}

function classNames(...values: Array<string | undefined>) {
  return values.filter(Boolean).join(' ');
}
