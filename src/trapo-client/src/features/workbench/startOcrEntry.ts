import type { NavigateFn } from '@tanstack/react-router';

import type { ModelAssetRecord } from '../../api/types';
import { addNotification } from '../../stores/notificationStore';

export function startOcrEntry(args: {
  model?: ModelAssetRecord;
  navigate: NavigateFn;
  selectedProfile: string;
}) {
  if (args.model?.status === 'downloaded') {
    void args.navigate({
      search: { model: args.model.model_id, profile: args.selectedProfile },
      to: '/ingest/start',
    });
    return;
  }
  addNotification({
    level: 'warning',
    message: args.model
      ? `${args.model.display_name} is selected but not downloaded. Download it before starting OCR.`
      : 'Select and download a model before starting OCR.',
    title: 'Model required for OCR',
  });
  void args.navigate({
    search: {
      model: args.model?.model_id,
      status: 'pending',
      view: 'grid',
    },
    to: '/models/downloads',
  });
}
