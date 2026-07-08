import type { TreeNode } from '../../components/workbench';
import type { WorkbenchExplorerScope } from './workbenchExplorerFilter';

interface SortableTreeNode extends TreeNode {
  children?: SortableTreeNode[];
  kind?: string;
  pageNo?: number;
}

export function sortNodes(node: SortableTreeNode) {
  node.children?.sort((left, right) => {
    const kindOrder = nodeKindOrder(left) - nodeKindOrder(right);
    if (kindOrder !== 0) {
      return kindOrder;
    }
    if (left.kind === 'page' && right.kind === 'page') {
      const pageOrder = pageNumberOrder(left, right);
      if (pageOrder !== 0) {
        return pageOrder;
      }
    }
    if (left.kind === 'engine' && right.kind === 'engine') {
      return 0;
    }
    return left.label.localeCompare(right.label);
  });
  node.children?.forEach(sortNodes);
}

export function pathParts(path: string) {
  return path.split(/[\\/]+/).filter(Boolean);
}

export function rootLabel(rootPath: string | undefined, scope: WorkbenchExplorerScope) {
  if (scope === 'all' && rootPath?.trim()) {
    return rootPath;
  }
  const parts = pathParts(rootPath ?? '');
  return parts.at(-1) ?? 'Workspace';
}

function nodeKindOrder(node: SortableTreeNode) {
  if (node.kind === 'folder') {
    return 0;
  }
  if (node.kind === 'document') {
    return 1;
  }
  if (node.kind === 'page') {
    return 2;
  }
  if (node.kind === 'engine') {
    return 3;
  }
  return 4;
}

function pageNumberOrder(left: SortableTreeNode, right: SortableTreeNode) {
  if (left.pageNo !== undefined && right.pageNo !== undefined) {
    return left.pageNo - right.pageNo;
  }
  if (left.pageNo !== undefined) {
    return -1;
  }
  if (right.pageNo !== undefined) {
    return 1;
  }
  return 0;
}
