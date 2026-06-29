class ApiError extends Error {
  readonly status: number;
  readonly statusText: string;

  constructor(message: string, response: Response) {
    super(message);
    this.name = 'ApiError';
    this.status = response.status;
    this.statusText = response.statusText;
  }
}

type ApiPath = `/api/${string}`;

export async function getJson<T>(url: string, signal?: AbortSignal): Promise<T> {
  const response = await fetch(toApiPath(url), { signal });
  await assertOk(response);
  return response.json() as Promise<T>;
}

export async function postJson<TResponse, TBody>(
  url: string,
  body: TBody,
  signal?: AbortSignal,
): Promise<TResponse> {
  const response = await fetch(toApiPath(url), {
    body: JSON.stringify(body),
    headers: { 'content-type': 'application/json' },
    method: 'POST',
    signal,
  });
  await assertOk(response);
  return response.json() as Promise<TResponse>;
}

export function buildApiUrl(path: ApiPath, params?: Record<string, string | number | undefined>) {
  const url = new URL(path, window.location.origin);
  for (const [key, value] of Object.entries(params ?? {})) {
    if (value !== undefined && value !== '') {
      url.searchParams.set(key, String(value));
    }
  }
  return toApiPath(`${url.pathname}${url.search}`);
}

function toApiPath(url: string): ApiPath {
  if (!url.startsWith('/api/') || url.startsWith('//') || url.includes('://')) {
    throw new ApiError(`Blocked non-local API URL: ${url}`, new Response(null, { status: 400 }));
  }
  return url as ApiPath;
}

async function assertOk(response: Response): Promise<void> {
  if (response.ok) {
    return;
  }
  const body = await response.text();
  throw new ApiError(body || `${response.status} ${response.statusText}`, response);
}
