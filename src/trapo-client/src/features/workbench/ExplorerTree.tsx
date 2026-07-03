import {
  CircleAlert,
  CircleCheck,
  Clock3,
  FileText,
  Folder,
  FolderOpen,
  LoaderCircle,
} from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';

import type { DocumentSummary } from '../../api/types';
import type { TreeNode } from '../../components/workbench';
import { TreeView } from '../../components/workbench';
import styles from './ExplorerTree.module.css';

interface ExplorerTreeProps {
  documents: DocumentSummary[];
  rootPath?: string;
  selectedFileHash?: string;
  onSelectDocument: (fileHash: string, pageNo?: number) => void;
}

export function ExplorerTree({
  documents,
  onSelectDocument,
  rootPath,
  selectedFileHash,
}: ExplorerTreeProps) {
  const tree = useMemo(
    () => buildDocumentTree(documents, onSelectDocument, selectedFileHash, rootPath),
    [documents, onSelectDocument, rootPath, selectedFileHash],
  );
  const [expandedIds, setExpandedIds] = useState(() => defaultExpandedIds(tree.nodes));

  useEffect(() => {
    setExpandedIds((current) => new Set([...current, ...defaultExpandedIds(tree.nodes)]));
  }, [tree.nodes]);

  return (
    <section className={styles.explorer} aria-label="Explorer">
      <div className={styles.header}>Explorer</div>
      <div className={styles.treeScroll}>
        {documents.length === 0 ? <div className={styles.empty}>No documents</div> : null}
        <TreeView
          className={styles.tree}
          expandedIds={expandedIds}
          nodes={tree.nodes}
          onToggle={(id) => toggleExpanded(id, setExpandedIds)}
        />
      </div>
    </section>
  );
}

interface MutableTreeNode extends TreeNode {
  children: MutableTreeNode[];
  kind: 'root' | 'folder' | 'document' | 'page';
  pageNo?: number;
}

export function buildDocumentTree(
  documents: DocumentSummary[],
  onSelectDocument: (fileHash: string, pageNo?: number) => void,
  selectedFileHash: string | undefined,
  rootPath: string | undefined,
) {
  const root: MutableTreeNode = {
    children: [],
    icon: <FolderOpen size={15} />,
    id: 'root',
    kind: 'root',
    label: rootLabel(rootPath),
  };
  const folders = new Map<string, MutableTreeNode>([['root', root]]);
  for (const document of sortedDocuments(documents)) {
    const parts = pathParts(document.relative_path || document.display_name);
    const fileName = parts.pop() || document.display_name;
    let parent = root;
    let folderId = 'root';
    for (const part of parts) {
      folderId = `${folderId}/${part}`;
      let folder = folders.get(folderId);
      if (!folder) {
        folder = {
          children: [],
          icon: <Folder size={15} />,
          id: folderId,
          kind: 'folder',
          label: part,
        };
        folders.set(folderId, folder);
        parent.children.push(folder);
      }
      parent = folder;
    }
    parent.children.push(documentNode(document, fileName, selectedFileHash, onSelectDocument));
  }
  sortNodes(root);
  return { nodes: [root] };
}

function documentNode(
  document: DocumentSummary,
  fileName: string,
  selectedDocumentId: string | undefined,
  onSelectDocument: (fileHash: string, pageNo?: number) => void,
): MutableTreeNode {
  const documentId = document.file_hash;
  const pageCount = Math.max(document.page_count || 1, 1);
  return {
    badge: <StatusIcon status={document.status} />,
    children:
      pageCount > 1
        ? Array.from({ length: pageCount }, (_, index) =>
            pageNode(document, documentId, index + 1, selectedDocumentId, onSelectDocument),
          )
        : [],
    icon: <FileText size={14} />,
    id: `document:${documentId}`,
    kind: 'document',
    label: fileName,
    onSelect: () => onSelectDocument(documentId, 1),
    selected: selectedDocumentId === documentId,
  };
}

function pageNode(
  document: DocumentSummary,
  documentId: string,
  pageNo: number,
  selectedDocumentId: string | undefined,
  onSelectDocument: (fileHash: string, pageNo?: number) => void,
): MutableTreeNode {
  return {
    badge: pageBadge(document, pageNo),
    children: [],
    icon: <FileText size={13} />,
    id: `document:${documentId}:page:${pageNo}`,
    kind: 'page',
    label: `Page ${pageNo}`,
    onSelect: () => onSelectDocument(documentId, pageNo),
    pageNo,
    selected: selectedDocumentId === documentId && document.current_page === pageNo,
  };
}

function StatusIcon({ status }: { status: string }) {
  if (status === 'completed') {
    return <CircleCheck className={styles.ok} size={13} />;
  }
  if (status === 'failed' || status === 'completed_with_errors') {
    return <CircleAlert className={styles.bad} size={13} />;
  }
  if (status === 'running' || status === 'rendering') {
    return <LoaderCircle className={styles.spin} size={13} />;
  }
  return <Clock3 className={styles.queued} size={13} />;
}

function pageBadge(document: DocumentSummary, pageNo: number) {
  if (document.current_page === pageNo && document.status === 'running') {
    return <LoaderCircle className={styles.spin} size={12} />;
  }
  if ((document.processed_pages ?? 0) >= pageNo) {
    return <CircleCheck className={styles.ok} size={12} />;
  }
  return <Clock3 className={styles.queued} size={12} />;
}

function sortedDocuments(documents: DocumentSummary[]) {
  return [...documents].sort((left, right) =>
    (left.relative_path || left.display_name).localeCompare(
      right.relative_path || right.display_name,
    ),
  );
}

function sortNodes(node: MutableTreeNode) {
  node.children.sort((left, right) => {
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
    return left.label.localeCompare(right.label);
  });
  for (const child of node.children) {
    sortNodes(child);
  }
}

function nodeKindOrder(node: MutableTreeNode) {
  if (node.kind === 'folder') {
    return 0;
  }
  if (node.kind === 'document') {
    return 1;
  }
  if (node.kind === 'page') {
    return 2;
  }
  return 3;
}

function pageNumberOrder(left: MutableTreeNode, right: MutableTreeNode) {
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

function defaultExpandedIds(nodes: TreeNode[]) {
  const ids = new Set<string>();
  const visit = (node: TreeNode, level: number) => {
    if (level < 2 && (node.children?.length ?? 0) > 0) {
      ids.add(node.id);
    }
    node.children?.forEach((child) => {
      visit(child, level + 1);
    });
  };
  nodes.forEach((node) => {
    visit(node, 0);
  });
  return ids;
}

function toggleExpanded(
  id: string,
  setExpandedIds: (update: (current: Set<string>) => Set<string>) => void,
) {
  setExpandedIds((current) => {
    const next = new Set(current);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    return next;
  });
}

function pathParts(path: string) {
  return path.split(/[\\/]+/).filter(Boolean);
}

function rootLabel(rootPath: string | undefined) {
  const parts = pathParts(rootPath ?? '');
  return parts.at(-1) ?? 'Workspace';
}
