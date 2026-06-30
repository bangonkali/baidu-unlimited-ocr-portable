import type { EventData, Step } from 'react-joyride';
import { Joyride, STATUS } from 'react-joyride';

import { setActiveView, setTourRun } from '../../stores/workbenchStore';

const steps: Step[] = [
  {
    content: 'Open Models first and download the Unlimited-OCR model files from Hugging Face.',
    target: '[data-tour="models"]',
  },
  {
    content: 'Choose a folder with PDFs or images. The manual path box is the fallback.',
    target: '[data-tour="folder"]',
  },
  {
    content: 'Start Scan queues supported files and processes PDF pages through bundled MuPDF.',
    target: '[data-tour="start"]',
  },
  {
    content:
      'Auto Follow keeps the newest OCR box in view. Turn it off when you want to inspect another region.',
    target: '[data-tour="preview"]',
  },
  {
    content: 'Diagnostics shows real runs and the server log file also printed in the terminal.',
    target: '[data-tour="diagnostics"]',
  },
];

interface GuidedTourProps {
  run: boolean;
}

export function GuidedTour({ run }: GuidedTourProps) {
  return (
    <Joyride
      continuous
      onEvent={handleTourEvent}
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

function handleTourEvent(event: EventData) {
  if (event.status === STATUS.FINISHED || event.status === STATUS.SKIPPED) {
    setTourRun(false);
    return;
  }
  if (event.index === 0) {
    setActiveView('models');
  }
  if (event.index === 1 || event.index === 2 || event.index === 3) {
    setActiveView('workbench');
  }
  if (event.index === 4) {
    setActiveView('diagnostics');
  }
}
