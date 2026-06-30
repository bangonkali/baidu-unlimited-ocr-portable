import { describe, expect, test } from 'bun:test';

import { fixtureModels } from '../../stories/fixtures/workbenchFixtures';
import { buildAppCommands, isActiveIngestState } from './appCommands';

describe('app command registry', () => {
  test('builds unique navigation, layout, theme, ingest, and model commands', () => {
    const commands = buildAppCommands({
      models: fixtureModels,
      panesCollapsed: { details: false, diagnostics: true, explorer: false },
      status: { active_run_id: null, default_profile: 'p', state: 'idle', supported_inputs: [] },
      theme: 'dark',
    });
    expect(new Set(commands.map((command) => command.id)).size).toBe(commands.length);
    expect(commands.some((command) => command.action.kind === 'togglePane')).toBe(true);
    expect(commands.some((command) => command.id === 'nav.ingest')).toBe(true);
    expect(commands.some((command) => command.id === 'model.open.unlimited-ocr-q4-k-m')).toBe(true);
  });

  test('detects one active ingest run at a time', () => {
    expect(
      isActiveIngestState({
        active_run_id: 'run-1',
        default_profile: 'p',
        state: 'running',
        supported_inputs: [],
      }),
    ).toBe(true);
    expect(
      isActiveIngestState({
        active_run_id: null,
        default_profile: 'p',
        state: 'idle',
        supported_inputs: [],
      }),
    ).toBe(false);
  });
});
