import { createContext, useContext } from 'react';

export interface DownloadsPaneContextValue {
  activeFileCount: number;
  close: () => void;
  isOpen: boolean;
  open: () => void;
  toggle: () => void;
}

const defaultDownloadsPane: DownloadsPaneContextValue = {
  activeFileCount: 0,
  close: () => undefined,
  isOpen: false,
  open: () => undefined,
  toggle: () => undefined,
};

export const DownloadsPaneContext = createContext<DownloadsPaneContextValue>(defaultDownloadsPane);

export function useDownloadsPane() {
  return useContext(DownloadsPaneContext);
}

interface DownloadModelInput {
  force?: boolean;
  modelId: string;
}

interface DownloadModelMutation {
  mutate: (input: DownloadModelInput) => void;
}

export function useDownloadModelWithPane(downloadModel: DownloadModelMutation) {
  const downloadsPane = useDownloadsPane();
  return {
    mutate: (input: DownloadModelInput) => {
      downloadsPane.open();
      downloadModel.mutate(input);
    },
  };
}
