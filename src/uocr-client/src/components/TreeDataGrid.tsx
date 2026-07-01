import { ChevronDown, ChevronRight } from 'lucide-react';
import type { ReactNode } from 'react';
import { useEffect, useMemo, useState } from 'react';

import styles from './TreeDataGrid.module.css';

export interface TreeDataGridNode {
  id: string;
  children?: TreeDataGridNode[];
}

export interface TreeDataGridColumn<TNode extends TreeDataGridNode> {
  id: string;
  header: string;
  width?: string;
  align?: 'left' | 'right';
  render: (node: TNode) => ReactNode;
}

interface TreeDataGridProps<TNode extends TreeDataGridNode> {
  ariaLabel: string;
  columns: TreeDataGridColumn<TNode>[];
  defaultExpandedDepth?: number;
  emptyLabel: string;
  nodes: TNode[];
}

interface FlatNode<TNode extends TreeDataGridNode> {
  depth: number;
  node: TNode;
}

export function TreeDataGrid<TNode extends TreeDataGridNode>({
  ariaLabel,
  columns,
  defaultExpandedDepth = 1,
  emptyLabel,
  nodes,
}: TreeDataGridProps<TNode>) {
  const defaultExpanded = useMemo(
    () => expandedIdsForDepth(nodes, defaultExpandedDepth),
    [defaultExpandedDepth, nodes],
  );
  const [expanded, setExpanded] = useState(defaultExpanded);
  const rows = useMemo(() => flattenNodes(nodes, expanded), [expanded, nodes]);
  const template = columns.map((column) => column.width ?? 'minmax(0, 1fr)').join(' ');

  useEffect(() => {
    setExpanded((current) => new Set([...current, ...defaultExpanded]));
  }, [defaultExpanded]);

  const toggle = (id: string) => {
    setExpanded((current) => {
      const next = new Set(current);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  return (
    <section className={styles.viewport} aria-label={ariaLabel}>
      <div className={styles.header} style={{ gridTemplateColumns: template }}>
        {columns.map((column) => (
          <div className={styles.headerCell} data-align={column.align} key={column.id}>
            {column.header}
          </div>
        ))}
      </div>
      <div className={styles.body}>
        {rows.length === 0 ? <div className={styles.empty}>{emptyLabel}</div> : null}
        {rows.map(({ depth, node }) => {
          const hasChildren = Boolean(node.children?.length);
          const isExpanded = expanded.has(node.id);
          return (
            <div className={styles.row} key={node.id} style={{ gridTemplateColumns: template }}>
              {columns.map((column, index) => (
                <div className={styles.cell} data-align={column.align} key={column.id}>
                  {index === 0 ? (
                    <div className={styles.treeCell} style={{ paddingLeft: `${depth * 16}px` }}>
                      {hasChildren ? (
                        <button
                          aria-expanded={isExpanded}
                          className={styles.disclosure}
                          onClick={() => toggle(node.id)}
                          title={isExpanded ? 'Collapse' : 'Expand'}
                          type="button"
                        >
                          {isExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                        </button>
                      ) : (
                        <span className={styles.disclosureSpacer} />
                      )}
                      {column.render(node)}
                    </div>
                  ) : (
                    column.render(node)
                  )}
                </div>
              ))}
            </div>
          );
        })}
      </div>
    </section>
  );
}

function expandedIdsForDepth<TNode extends TreeDataGridNode>(nodes: TNode[], depth: number) {
  const expanded = new Set<string>();
  const visit = (items: TreeDataGridNode[], level: number) => {
    if (level >= depth) {
      return;
    }
    for (const node of items) {
      if (node.children?.length) {
        expanded.add(node.id);
        visit(node.children, level + 1);
      }
    }
  };
  visit(nodes, 0);
  return expanded;
}

function flattenNodes<TNode extends TreeDataGridNode>(nodes: TNode[], expanded: Set<string>) {
  const rows: Array<FlatNode<TNode>> = [];
  const visit = (items: TNode[], depth: number) => {
    for (const node of items) {
      rows.push({ depth, node });
      if (expanded.has(node.id) && node.children?.length) {
        visit(node.children as TNode[], depth + 1);
      }
    }
  };
  visit(nodes, 0);
  return rows;
}
