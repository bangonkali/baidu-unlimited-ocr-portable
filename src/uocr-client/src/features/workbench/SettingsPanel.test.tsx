import { describe, expect, test } from 'bun:test';
import { renderToString } from 'react-dom/server';

import { fixtureModels, fixtureSettings } from '../../stories/fixtures/workbenchFixtures';
import { SettingsPanel } from './SettingsPanel';

describe('SettingsPanel', () => {
  test('renders runtime choices and disables unsupported accelerators', () => {
    const html = renderToString(
      <SettingsPanel
        models={fixtureModels}
        onModelChange={() => undefined}
        onProfileChange={() => undefined}
        onRuntimeChange={() => undefined}
        profiles={fixtureModels.profiles}
        selectedProfile="experimental-exact-prefill-q4"
        settings={fixtureSettings}
      />,
    );

    expect(html).toContain('Windows x64 CUDA 13');
    expect(html).toContain('Windows x64 AMD ROCm/HIP');
    expect(html).toContain('Runtime files are not installed');
    expect(html).toContain('Unlimited-OCR Q4_K_M');
    expect(html).toContain('Experimental exact-prefill Q4');
    expect(html).toContain('disabled=""');
  });
});
