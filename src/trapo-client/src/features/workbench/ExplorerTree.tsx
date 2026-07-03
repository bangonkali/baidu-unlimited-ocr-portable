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
}

function buildDocumentTree(
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
  selectedFileHash: string | undefined,
  onSelectDocument: (fileHash: string, pageNo?: number) => void,
): MutableTreeNode {
  const pageCount = Math.max(document.page_count || 1, 1);
  return {
    badge: <StatusIcon status={document.status} />,
    children:
      pageCount > 1
        ? Array.from({ length: pageCount }, (_, index) => pageNode(document, index + 1))
        : [],
    icon: <FileText size={14} />,
    id: `document:${document.file_hash}`,
    kind: 'document',
    label: fileName,
    onSelect: () => onSelectDocument(document.file_hash, 1),
    selected: selectedFileHash === document.file_hash,
  };

  function pageNode(source: DocumentSummary, pageNo: number): MutableTreeNode {
    return {
      badge: pageBadge(source, pageNo),
      children: [],
      icon: <FileText size={13} />,
      id: `document:${source.file_hash}:page:${pageNo}`,
      kind: 'page',
      label: `Page ${pageNo}`,
      onSelect: () => onSelectDocument(source.file_hash, pageNo),
      selected: selectedFileHash === source.file_hash && source.current_page === pageNo,
    };
  }
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
    if (left.kind !== right.kind) {
      return left.kind === 'folder' ? -1 : 1;
    }
    return left.label.localeCompare(right.label);
  });
  for (const child of node.children) {
    sortNodes(child);
  }
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
