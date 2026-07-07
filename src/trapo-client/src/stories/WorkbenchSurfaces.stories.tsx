import type { Meta, StoryObj } from '@storybook/react-vite';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { RouterContextProvider } from '@tanstack/react-router';

import { DetailsPane } from '../features/workbench/DetailsPane';
import { DiagnosticsPanel } from '../features/workbench/DiagnosticsPanel';
import { ExplorerTree } from '../features/workbench/ExplorerTree';
import { IngestStartPanel } from '../features/workbench/IngestStartPanel';
import { ModelDetailPanel } from '../features/workbench/ModelDetailPanel';
import { ModelManager } from '../features/workbench/ModelManager';
import { PreviewPane } from '../features/workbench/PreviewPane';
import { SearchPane } from '../features/workbench/SearchPane';
import { SettingsPanel } from '../features/workbench/SettingsPanel';
import { StartHere } from '../features/workbench/StartHere';
import { TextPane } from '../features/workbench/TextPane';
import { router } from '../router';
import {
  fixtureBoxes,
  fixtureDocuments,
  fixtureDownloadingModels,
  fixtureLogs,
  fixtureModels,
  fixturePages,
  fixtureRuns,
  fixtureSettings,
} from './fixtures/workbenchFixtures';
import {
  fixtureSearchFiles,
  fixtureSearchHits,
  fixtureUsedEmbeddingModels,
} from './fixtures/workbenchSearchFixtures';
import './storybook.css';

const meta = {
  decorators: [
    (Story) => (
      <RouterContextProvider router={router}>
        <Story />
      </RouterContextProvider>
    ),
  ],
  title: 'Workbench/Surfaces',
} satisfies Meta;

export default meta;

type Story = StoryObj<typeof meta>;

const storyQueryClient = new QueryClient({
  defaultOptions: { queries: { enabled: false, retry: false } },
});

export const Ingest: Story = {
  render: () => (
    <div className="storyTall">
      <IngestStartPanel
        activeRun={fixtureRuns[0]}
        model={fixtureModels.models[0]}
        models={fixtureModels}
        onCancelModel={() => undefined}
        onDownloadModel={() => undefined}
        onModelChange={() => undefined}
        onPickFolder={() => undefined}
        onGenerateEmbedding={() => undefined}
        onProfileChange={() => undefined}
        onRootPathChange={() => undefined}
        onStart={() => undefined}
        onStartTextIndex={() => undefined}
        onStop={() => undefined}
        profiles={fixtureModels.profiles}
        rootPath="C:\\data\\incoming"
        runs={fixtureRuns}
        selectedProfile="experimental-exact-prefill-q4"
        status={{
          default_profile: 'experimental-exact-prefill-q4',
          state: 'running',
          supported_inputs: [],
        }}
      />
    </div>
  ),
};

export const Explorer: Story = {
  render: () => (
    <div className="storyTall">
      <ExplorerTree
        documents={fixtureDocuments}
        filter={{ runId: fixtureRuns[0]?.run_id, scope: 'run' }}
        onFilterChange={() => undefined}
        onSelectDocument={() => undefined}
        runs={fixtureRuns}
        selectedRunId={fixtureRuns[0]?.run_id}
        selectedFileHash="hash-invoice-014"
      />
    </div>
  ),
};

export const Search: Story = {
  render: () => (
    <div className="storyTall">
      <SearchPane
        documents={new Map(fixtureDocuments.map((document) => [document.file_hash, document]))}
        files={fixtureSearchFiles}
        hits={fixtureSearchHits}
        loading={false}
        models={fixtureUsedEmbeddingModels}
        query="asuka"
        runs={fixtureRuns}
        selectedModelId="nomic-embed-text-v1-5-q4-k-m"
        view="tree"
        onChange={() => undefined}
        onSelectHit={() => undefined}
      />
    </div>
  ),
};

export const Traceability: Story = {
  render: () => (
    <div className="storySplit">
      <PreviewPane
        autoFollowRegions
        boxes={fixtureBoxes}
        fileHash="hash-invoice-014"
        getImageUrl={() => fixturePreviewImage}
        labelsVisible
        overlayVisible
        pages={[1]}
        selectedPageNo={1}
        selectedRegionId="reg-total"
        onAutoFollowChange={() => undefined}
        onSelectRegion={() => undefined}
      />
      <TextPane
        autoFollowRegions
        document={fixtureDocuments[0]}
        onSelectRegion={() => undefined}
        pages={fixturePages}
        regions={fixtureBoxes}
        selectedRegionId="reg-total"
      />
    </div>
  ),
};

export const LiveText: Story = {
  render: () => (
    <div className="storyTall">
      <TextPane
        autoFollowRegions
        document={fixtureDocuments[1]}
        onSelectRegion={() => undefined}
        pages={fixturePages}
        regions={fixtureBoxes}
        selectedRegionId="reg-supplier"
      />
    </div>
  ),
};

export const Details: Story = {
  render: () => (
    <div className="storyTall">
      <DetailsPane
        document={fixtureDocuments[0]}
        labelsVisible
        overlayVisible
        selectedRegion={fixtureBoxes[0]}
        selectedRegionId="reg-total"
      />
    </div>
  ),
};

export const Diagnostics: Story = {
  render: () => (
    <div className="storyTall">
      <QueryClientProvider client={storyQueryClient}>
        <DiagnosticsPanel logs={fixtureLogs} runs={fixtureRuns} search={{ tab: 'logs' }} />
      </QueryClientProvider>
    </div>
  ),
};

export const Models: Story = {
  render: () => (
    <div className="storyTall">
      <ModelManager
        models={fixtureModels}
        onCancelModel={() => undefined}
        onDownloadModel={() => undefined}
        onSelectModel={() => undefined}
      />
    </div>
  ),
};

export const ModelsDownloading: Story = {
  render: () => (
    <div className="storyTall">
      <ModelManager
        models={fixtureDownloadingModels}
        onCancelModel={() => undefined}
        onDownloadModel={() => undefined}
        onSelectModel={() => undefined}
      />
    </div>
  ),
};

export const ModelDownloads: Story = {
  render: () => (
    <div className="storyTall">
      <ModelManager
        models={fixtureDownloadingModels}
        onCancelModel={() => undefined}
        onDownloadModel={() => undefined}
        onSelectModel={() => undefined}
        routeSearch={{ status: 'all', view: 'grid' }}
        scope="downloads"
      />
    </div>
  ),
};

export const ModelDetail: Story = {
  render: () => (
    <div className="storyTall">
      <ModelDetailPanel
        model={fixtureModels.models[0]}
        onCancelModel={() => undefined}
        onDownloadModel={() => undefined}
        onSelectModel={() => undefined}
      />
    </div>
  ),
};

export const Settings: Story = {
  render: () => (
    <div className="storyTall">
      <SettingsPanel
        models={fixtureModels}
        onDownloadConcurrencyChange={() => undefined}
        onModelChange={() => undefined}
        onProfileChange={() => undefined}
        onRuntimeChange={() => undefined}
        onThemeChange={() => undefined}
        profiles={fixtureModels.profiles}
        selectedProfile="experimental-exact-prefill-q4"
        settings={fixtureSettings}
        theme="dark"
      />
    </div>
  ),
};

export const Start: Story = {
  render: () => (
    <div className="storyFrame">
      <StartHere
        model={{
          display_name: 'Unlimited-OCR Q4_K_M',
          model_id: 'unlimited-ocr-q4-k-m',
          status: 'missing',
        }}
        onOpenModels={() => undefined}
        onPickFolder={() => undefined}
        onStart={() => undefined}
        rootPath=""
      />
    </div>
  ),
};

const fixturePreviewImage =
  'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="720" height="960"><rect width="720" height="960" fill="%23f7f7f2"/><text x="96" y="160" font-family="Segoe UI" font-size="36" fill="%23222">Supplier</text><text x="130" y="360" font-family="Segoe UI" font-size="34" fill="%23222">Invoice total: 1,240.00</text></svg>';
