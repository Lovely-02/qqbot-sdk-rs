import { Buffer } from 'node:buffer';

// Match the core SDK's image size limit before allocating or downloading the payload.
const maxMaterialBytes = 10 * 1024 * 1024;

export type UploadSource = Buffer | string | URL;

export function parseUploadSource(source: UploadSource, maxBytes?: number): Buffer | URL {
  if (Buffer.isBuffer(source)) {
    checkSize(source.length, maxBytes);
    // Snapshot the bytes before the asynchronous native upload.
    return Buffer.from(source);
  }
  if (source instanceof URL || (typeof source === 'string' && /^https?:\/\//i.test(source))) {
    const url = new URL(source);
    if (url.protocol !== 'http:' && url.protocol !== 'https:') {
      throw new TypeError('Upload URL must use HTTP or HTTPS');
    }
    return url;
  }
  if (typeof source !== 'string') {
    throw new TypeError('Upload source must be a Buffer, Base64 string or HTTP(S) URL');
  }

  return decodeBase64(source, maxBytes);
}

export async function readMaterialSource(source: UploadSource): Promise<Buffer> {
  const parsed = parseUploadSource(source, maxMaterialBytes);
  return parsed instanceof URL ? downloadMaterial(parsed) : parsed;
}

function decodeBase64(source: string, maxBytes?: number): Buffer {
  const base64 = source.trim().replace(/^data:[\w.+-]+\/[\w.+-]+;base64,/i, '').replace(/[\t\n\r ]/g, '');
  if (maxBytes !== undefined && base64.length > Math.ceil(maxBytes / 3) * 4) {
    throw new RangeError(`Decoded upload exceeds ${maxBytes} bytes`);
  }
  const bytes = Buffer.from(base64, 'base64');
  const encoded = bytes.toString('base64');
  if (!bytes.length || (base64 !== encoded && base64 !== encoded.replace(/=+$/, ''))) {
    throw new TypeError('Invalid Base64 upload data');
  }
  checkSize(bytes.length, maxBytes);
  return bytes;
}

function checkSize(size: number, maxBytes?: number): void {
  if (maxBytes !== undefined && size > maxBytes) {
    throw new RangeError(`Upload data must not exceed ${maxBytes} bytes`);
  }
}

async function downloadMaterial(url: URL): Promise<Buffer> {
  const response = await fetch(url, { signal: AbortSignal.timeout(30_000) });
  if (!response.ok || !response.body) {
    await response.body?.cancel();
    throw new Error(`Material download failed (HTTP ${response.status})`);
  }

  const reader = response.body.getReader();
  const chunks: Uint8Array[] = [];
  let size = 0;
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      size += value.byteLength;
      checkSize(size, maxMaterialBytes);
      chunks.push(value);
    }
  } catch (error) {
    await reader.cancel().catch(() => {});
    throw error;
  } finally {
    reader.releaseLock();
  }
  return Buffer.concat(chunks, size);
}
