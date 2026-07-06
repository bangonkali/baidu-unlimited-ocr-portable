import { AlertTriangle, CheckCircle2, CircleDot } from 'lucide-react';

import styles from './IngestWizard.module.css';

export interface WizardStepRecord {
  description: string;
  label: string;
  status: 'blocked' | 'current' | 'pending' | 'ready';
}

export function IngestWizardStepper({ steps }: { steps: WizardStepRecord[] }) {
  return (
    <nav className={styles.stepper} aria-label="Ingest workflow">
      {steps.map((step, index) => (
        <section className={styles.step} data-status={step.status} key={step.label}>
          <span className={styles.stepIcon}>{stepIcon(step.status)}</span>
          <span className={styles.stepText}>
            <strong>
              {index + 1}. {step.label}
            </strong>
            <span>{step.description}</span>
          </span>
        </section>
      ))}
    </nav>
  );
}

function stepIcon(status: WizardStepRecord['status']) {
  if (status === 'ready') {
    return <CheckCircle2 size={14} />;
  }
  if (status === 'blocked') {
    return <AlertTriangle size={14} />;
  }
  return <CircleDot size={14} />;
}
