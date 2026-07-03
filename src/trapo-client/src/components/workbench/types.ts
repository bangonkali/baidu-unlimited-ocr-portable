import type { ReactNode } from 'react';

export interface TreeNode {
  id: string;
  label: string;
  icon?: ReactNode;
  badge?: ReactNode;
  checked?: boolean | 'indeterminate';
  hasChildren?: boolean;
  selected?: boolean;
  children?: TreeNode[];
  onExpand?: () => void;
  onSelect?: () => void;
  onCheckedChange?: (checked: boolean) => void;
}

export interface TreeGridNode extends Omit<TreeNode, 'children'> {
  children?: TreeGridNode[];
}
