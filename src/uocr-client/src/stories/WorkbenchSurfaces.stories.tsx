import type { Meta, StoryObj } from '@storybook/react-vite';

import { DetailsPane } from '../features/workbench/DetailsPane';
import { DiagnosticsPanel } from '../features/workbench/DiagnosticsPanel';
import { ExplorerTree } from '../features/workbench/ExplorerTree';
import { IngestToolbar } from '../features/workbench/IngestToolbar';
import { PreviewPane } from '../features/workbench/PreviewPane';
import { TextPane } from '../features/workbench/TextPane';
import {
  fixtureBoxes,
  fixtureDocuments,
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
        onPause={() => undefined}
        onPickFolder={() => undefined}
        onProfileChange={() => undefined}
        onRootPathChange={() => undefined}
        onStart={() => undefined}
        onStop={() => undefined}
        profiles={fixtureModels.profiles}
        rootPath="C:\\data\\incoming"
        selectedProfile="best-zero-empty-q4"
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
      <PreviewPane boxes={fixtureBoxes} labelsVisible overlayVisible selectedRegionId="reg-total" />
      <TextPane pages={fixturePages} selectedRegionId="reg-total" />
    </div>
  ),
};

export const Details: Story = {
  render: () => (
    <div className="storyTall">
      <DetailsPane
        labelsVisible
        models={fixtureModels}
        overlayVisible
        selectedFileHash="hash-invoice-014"
        selectedRegionId="reg-total"
        settings={fixtureSettings}
      />
    </div>
  ),
};

export const Diagnostics: Story = {
  render: () => (
    <div className="storyTall">
      <DiagnosticsPanel runs={fixtureRuns} />
    </div>
  ),
};
