import type { EventData, Step } from 'react-joyride';
import { Joyride, STATUS } from 'react-joyride';

import type { ActiveView } from '../../stores/workbenchStore';
import { setTourRun } from '../../stores/workbenchStore';

const steps: Step[] = [
  {
    content:
      'Use Start OCR to begin. If the selected model is missing, Trapo opens the downloader.',
    target: '[data-tour="start-ocr"]',
  },
  {
    content: 'Download and select a local OCR model before starting an ingest run.',
    target: '[data-tour="models"]',
  },
  {
    content: 'Choose a folder with PDFs or images. The manual path box is the fallback.',
    target: '[data-tour="folder"]',
  },
  {
    content: 'Start Scan queues supported files and processes PDF pages through bundled PDFium.',
    target: '[data-tour="start"]',
  },
  {
    content:
      'Auto Follow keeps the newest OCR box in view. Turn it off when you want to inspect another region.',
    target: '[data-tour="preview"]',
  },
  {
    content:
      'The explorer follows the target directory tree and marks queued, running, and completed pages.',
    target: '[aria-label="Explorer"]',
  },
  {
    content: 'Diagnostics shows the OCR waterfall, work units, model leases, and logs.',
    target: '[data-tour="diagnostics"]',
  },
];

interface GuidedTourProps {
  run: boolean;
  onViewChange: (view: ActiveView) => void;
}

export function GuidedTour({ onViewChange, run }: GuidedTourProps) {
  return (
    <Joyride
      continuous
      onEvent={(event) => handleTourEvent(event, onViewChange)}
      options={{
        arrowColor: '#252526',
        backgroundColor: '#252526',
        buttons: ['back', 'close', 'primary', 'skip'],
        overlayClickAction: false,
        primaryColor: '#4cc2ff',
        showProgress: true,
        textColor: '#e7e7e7',
        zIndex: 10000,
      }}
      run={run}
      scrollToFirstStep
      steps={steps}
    />
  );
}

function handleTourEvent(event: EventData, onViewChange: (view: ActiveView) => void) {
  if (event.status === STATUS.FINISHED || event.status === STATUS.SKIPPED) {
    setTourRun(false);
    return;
  }
  if (event.index === 1) {
    onViewChange('models');
  }
  if (event.index === 2 || event.index === 3) {
    onViewChange('ingest');
  }
  if (event.index === 0 || event.index === 4 || event.index === 5) {
    onViewChange('workbench');
  }
  if (event.index === 6) {
    onViewChange('diagnostics');
  }
}
