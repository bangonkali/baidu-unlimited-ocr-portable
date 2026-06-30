import type { Meta, StoryObj } from '@storybook/react-vite';

import { DetailsPane } from '../features/workbench/DetailsPane';
import { DiagnosticsPanel } from '../features/workbench/DiagnosticsPanel';
import { ExplorerTree } from '../features/workbench/ExplorerTree';
import { IngestToolbar } from '../features/workbench/IngestToolbar';
import { ModelManager } from '../features/workbench/ModelManager';
import { PreviewPane } from '../features/workbench/PreviewPane';
import { SettingsPanel } from '../features/workbench/SettingsPanel';
import { StartHere } from '../features/workbench/StartHere';
import { TextPane } from '../features/workbench/TextPane';
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
import './storybook.css';

const meta = {
  title: 'Workbench/Surfaces',
} satisfies Meta;

export default meta;

type Story = StoryObj<typeof meta>;

export const Ingest: Story = {
  render: () => (
    <div className="storyFrame">
      <IngestToolbar
        activeRun={fixtureRuns[0]}
        modelReady
        onRefresh={() => undefined}
        onPickFolder={() => undefined}
        onProfileChange={() => undefined}
        onRootPathChange={() => undefined}
        onStart={() => undefined}
        onStop={() => undefined}
        profiles={fixtureModels.profiles}
        rootPath="C:\\data\\incoming"
        runState="idle"
        selectedProfile="experimental-exact-prefill-q4"
        supportedInputs={['.pdf', '.png', '.jpg', '.webp']}
      />
    </div>
  ),
};

export const Explorer: Story = {
  render: () => (
    <div className="storyTall">
      <ExplorerTree documents={fixtureDocuments} selectedFileHash="hash-invoice-014" />
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
      />
      <TextPane pages={fixturePages} selectedRegionId="reg-total" />
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
      <DiagnosticsPanel logs={fixtureLogs} runs={fixtureRuns} />
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

export const Settings: Story = {
  render: () => (
    <div className="storyTall">
      <SettingsPanel
        models={fixtureModels}
        onModelChange={() => undefined}
        onProfileChange={() => undefined}
        onRuntimeChange={() => undefined}
        profiles={fixtureModels.profiles}
        selectedProfile="experimental-exact-prefill-q4"
        settings={fixtureSettings}
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
