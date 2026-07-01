import { describe, expect, test } from 'bun:test';
import { renderToString } from 'react-dom/server';

import { fixtureModels, fixtureSettings } from '../../stories/fixtures/workbenchFixtures';
import { SettingsPanel } from './SettingsPanel';

describe('SettingsPanel', () => {
  test('renders runtime choices and disables unsupported accelerators', () => {
    const appearanceHtml = renderSettingsPanel();
    const runtimeHtml = renderSettingsPanel('runtime');
    const ocrHtml = renderSettingsPanel('ocr');

    expect(appearanceHtml).toContain('Appearance');
    expect(runtimeHtml).toContain('Windows x64 CUDA 13');
    expect(runtimeHtml).toContain('Windows x64 AMD ROCm/HIP');
    expect(runtimeHtml).toContain('Runtime files are not installed');
    expect(ocrHtml).toContain('Unlimited-OCR Q4_K_M');
    expect(ocrHtml).toContain('Experimental exact-prefill Q4');
    expect(runtimeHtml).toContain('disabled=""');
  });
});

function renderSettingsPanel(activeSection?: 'runtime' | 'ocr') {
  return renderToString(
    <SettingsPanel
      activeSection={activeSection}
      models={fixtureModels}
      onModelChange={() => undefined}
      onProfileChange={() => undefined}
      onRuntimeChange={() => undefined}
      onThemeChange={() => undefined}
      profiles={fixtureModels.profiles}
      selectedProfile="experimental-exact-prefill-q4"
      settings={fixtureSettings}
      theme="dark"
    />,
  );
}
