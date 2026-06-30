import type { ModelAssetRecord, ModelDownloadFileRecord } from '../../api/types';
import fileStyles from './ModelFileTable.module.css';
import { formatBytes, formatEta, formatPercent, formatRate } from './modelDownloadFormat';

export function ModelFileTable({
  files,
  model,
}: {
  files?: ModelDownloadFileRecord[];
  model: ModelAssetRecord;
}) {
  return (
    <table className={fileStyles.files} aria-label="Required model files">
      <thead>
        <tr className={fileStyles.fileHeader}>
          <th scope="col">File</th>
          <th scope="col">Status</th>
          <th scope="col">Progress</th>
          <th scope="col">Rate</th>
          <th scope="col">ETA</th>
        </tr>
      </thead>
      <tbody>
        {(files ?? fallbackFiles(model)).map((file) => (
          <tr className={fileStyles.fileRow} key={file.file_id}>
            <td title={file.file_name}>{file.file_name}</td>
            <td>{file.status}</td>
            <td>
              {formatBytes(file.downloaded_bytes)} / {formatBytes(file.total_bytes)} (
              {formatPercent(file.percent)})
            </td>
            <td>{formatRate(file.bytes_per_second)}</td>
            <td>{formatEta(file.eta_seconds)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

export function fallbackFiles(model: ModelAssetRecord): ModelDownloadFileRecord[] {
  return [model.model_file, model.mmproj_file].filter(Boolean).map((fileName, index) => ({
    downloaded_bytes: 0,
    file_id: index === 0 ? 'model' : 'mmproj',
    file_name: fileName ?? '',
    percent: 0,
    status: model.status,
  }));
}
