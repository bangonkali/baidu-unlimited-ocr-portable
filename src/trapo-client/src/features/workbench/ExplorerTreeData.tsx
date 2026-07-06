import {
  CircleAlert,
  CircleCheck,
  Clock3,
  FileText,
  Folder,
  FolderOpen,
  LoaderCircle,
} from 'lucide-react';

import type { DocumentSummary, IngestRunRecord } from '../../api/types';
import type { TreeNode } from '../../components/workbench';
import styles from './ExplorerTree.module.css';
import type { WorkbenchExplorerScope } from './workbenchExplorerFilter';

export interface ExplorerTreeBuildOptions {
  documents: DocumentSummary[];
  fallbackRootPath?: string;
  onSelectDocument: (fileHash: string, pageNo?: number, runId?: string) => void;
  runId?: string;
  runs: IngestRunRecord[];
  scope: WorkbenchExplorerScope;
  selectedFileHash?: string;
  selectedRunId?: string;
}

interface MutableTreeNode extends TreeNode {
  children: MutableTreeNode[];
  kind: 'root' | 'folder' | 'document' | 'page';
  pageNo?: number;
}

interface TreeRootSource {
  documents: DocumentSummary[];
  id: string;
  rootPath?: string;
  runId?: string;
}

export function buildDocumentTree(options: ExplorerTreeBuildOptions) {
  const sources = treeRootSources(options);
  const roots = sources.map((source) => rootNode(source, options));
  for (const root of roots) {
    sortNodes(root);
  }
  return {
    documentCount: sources.reduce((count, source) => count + source.documents.length, 0),
    nodes: roots,
  };
}

function treeRootSources(options: ExplorerTreeBuildOptions): TreeRootSource[] {
  if (options.runs.length === 0) {
    return [
      {
        documents: sortedDocuments(options.documents),
        id: 'workspace',
        rootPath: options.fallbackRootPath,
      },
    ];
  }
  const selectedRuns =
    options.scope === 'all'
      ? options.runs
      : options.runs.filter((run) => run.run_id === options.runId);
  if (selectedRuns.length === 0 && options.scope === 'run' && options.runId) {
    return [{ documents: [], id: options.runId, rootPath: options.fallbackRootPath }];
  }
  const documentsByHash = new Map(
    options.documents.map((document) => [document.file_hash, document]),
  );
  return selectedRuns.map((run) => ({
    documents: sortedDocuments(documentsForRun(run, documentsByHash)),
    id: run.run_id,
    rootPath: run.root_path,
    runId: run.run_id,
  }));
}

function rootNode(source: TreeRootSource, options: ExplorerTreeBuildOptions): MutableTreeNode {
  const root: MutableTreeNode = {
    children: [],
    icon: <FolderOpen size={15} />,
    id: `root:${source.id}`,
    kind: 'root',
    label: rootLabel(source.rootPath, options.scope),
  };
  const folders = new Map<string, MutableTreeNode>([[root.id, root]]);
  for (const document of source.documents) {
    addDocumentNode(root, folders, document, source.runId, options);
  }
  return root;
}

function addDocumentNode(
  root: MutableTreeNode,
  folders: Map<string, MutableTreeNode>,
  document: DocumentSummary,
  runId: string | undefined,
  options: ExplorerTreeBuildOptions,
) {
  const parts = pathParts(document.relative_path || document.display_name);
  const fileName = parts.pop() || document.display_name;
  let parent = root;
  let folderId = root.id;
  for (const part of parts) {
    folderId = `${folderId}/folder:${part}`;
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
  parent.children.push(documentNode(document, fileName, runId, options));
}

function documentsForRun(
  run: IngestRunRecord,
  documentsByHash: Map<string, DocumentSummary>,
): DocumentSummary[] {
  const seen = new Set<string>();
  const documents: DocumentSummary[] = [];
  for (const hash of run.file_hashes ?? []) {
    if (seen.has(hash)) {
      continue;
    }
    seen.add(hash);
    const document = documentsByHash.get(hash);
    if (document) {
      documents.push(document);
    }
  }
  return documents;
}

function documentNode(
  document: DocumentSummary,
  fileName: string,
  runId: string | undefined,
  options: ExplorerTreeBuildOptions,
): MutableTreeNode {
  const documentId = document.file_hash;
  const pageCount = Math.max(document.page_count || 1, 1);
  const selected = isSelectedDocument(documentId, runId, options);
  return {
    badge: <StatusIcon status={document.status} />,
    children:
      pageCount > 1
        ? Array.from({ length: pageCount }, (_, index) =>
            pageNode(document, runId, index + 1, selected, options),
          )
        : [],
    icon: <FileText size={14} />,
    id: nodeId('document', runId, documentId),
    kind: 'document',
    label: fileName,
    onSelect: () => options.onSelectDocument(documentId, 1, runId),
    selected,
  };
}

function pageNode(
  document: DocumentSummary,
  runId: string | undefined,
  pageNo: number,
  documentSelected: boolean,
  options: ExplorerTreeBuildOptions,
): MutableTreeNode {
  return {
    badge: pageBadge(document, pageNo),
    children: [],
    icon: <FileText size={13} />,
    id: nodeId('page', runId, document.file_hash, pageNo),
    kind: 'page',
    label: `Page ${pageNo}`,
    onSelect: () => options.onSelectDocument(document.file_hash, pageNo, runId),
    pageNo,
    selected: documentSelected && document.current_page === pageNo,
  };
}

function isSelectedDocument(
  documentId: string,
  runId: string | undefined,
  options: Pick<ExplorerTreeBuildOptions, 'selectedFileHash' | 'selectedRunId'>,
) {
  return (
    options.selectedFileHash === documentId && // skylos: ignore[SKY-D253] file_hash is a public workbench selection identifier, not a secret token.
    (options.selectedRunId === undefined || options.selectedRunId === runId) // skylos: ignore[SKY-D253] run_id is public route/UI state, not a secret token.
  );
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

function nodeId(
  kind: 'document' | 'page',
  runId: string | undefined,
  fileHash: string,
  pageNo?: number,
) {
  const runPart = runId ?? 'workspace';
  const base = `run:${runPart}:document:${fileHash}`;
  return kind === 'document' ? base : `${base}:page:${pageNo}`;
}

function pathParts(path: string) {
  return path.split(/[\\/]+/).filter(Boolean);
}

function rootLabel(rootPath: string | undefined, scope: WorkbenchExplorerScope) {
  if (scope === 'all' && rootPath?.trim()) {
    return rootPath;
  }
  const parts = pathParts(rootPath ?? '');
  return parts.at(-1) ?? 'Workspace';
}
