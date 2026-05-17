import vokexCall from "./vokexCall";
import { events } from "./events";

// ─── 类型定义 ────────────────────────────────────────────

/**
 * HTTP 请求选项（扩展浏览器 RequestInit，增加 stream 和 timeout）
 */
export interface HttpInit extends Omit<RequestInit, 'body'> {
  /** 请求体（支持 string / 纯对象 / FormData / 标准 BodyInit） */
  body?: any;
  /** 是否启用流式（SSE）模式 */
  stream?: boolean;
  /** 超时时间（秒） */
  timeout?: number;
}

/**
 * HTTP API 接口
 */
export interface HttpAPI {
  /**
   * 核心 HTTP 请求（兼容浏览器 fetch，扩展支持 stream/timeout）
   *
   * @example
   * // 普通请求
   * const resp = await http.fetch('https://api.example.com/data');
   * const data = await resp.json();
   *
   * // POST JSON（纯对象自动序列化）
   * const resp = await http.fetch('https://api.example.com/users', {
   *   method: 'POST',
   *   body: { name: 'Alice' },
   * });
   *
   * // SSE 流式请求
   * const resp = await http.fetch('https://api.example.com/chat', {
   *   method: 'POST',
   *   body: { prompt: 'Hello' },
   *   stream: true,
   * });
   * const reader = resp.body!.getReader();
   */
  fetch(url: string | URL, init?: HttpInit): Promise<Response>;

  /** GET 请求语法糖 */
  get(url: string, init?: Omit<HttpInit, 'method' | 'body'>): Promise<Response>;

  /** POST 请求语法糖 */
  post(url: string, body?: any, init?: Omit<HttpInit, 'method' | 'body'>): Promise<Response>;

  /** PUT 请求语法糖 */
  put(url: string, body?: any, init?: Omit<HttpInit, 'method' | 'body'>): Promise<Response>;

  /** DELETE 请求语法糖 */
  delete(url: string, init?: Omit<HttpInit, 'method' | 'body'>): Promise<Response>;
}

// ─── 内部工具函数 ────────────────────────────────────────

/**
 * 判断是否为纯对象（排除 Array、Date、RegExp、null 等）
 */
function isPlainObject(val: unknown): val is Record<string, any> {
  if (val === null || typeof val !== 'object') return false;
  const proto = Object.getPrototypeOf(val);
  return proto === Object.prototype || proto === null;
}

/**
 * 检查 headers 中是否已存在指定 key（不区分大小写）
 */
function hasHeader(headers: Record<string, string> | undefined, key: string): boolean {
  if (!headers) return false;
  const lower = key.toLowerCase();
  for (const k in headers) {
    if (Object.prototype.hasOwnProperty.call(headers, k) && k.toLowerCase() === lower) {
      return true;
    }
  }
  return false;
}

/**
 * 将 FormData 转换为 URL-encoded 字符串
 */
function formDataToUrlEncoded(formData: FormData): string {
  const pairs: string[] = [];
  formData.forEach((value, key) => {
    pairs.push(`${encodeURIComponent(key)}=${encodeURIComponent(String(value))}`);
  });
  return pairs.join('&');
}

/**
 * 智能处理请求体和请求头
 * - Plain Object → JSON.stringify + 自动 Content-Type
 * - FormData → URL-encoded + 自动 Content-Type
 * - string / undefined → 原样传递
 */
function resolveBody(body: any, headers?: Record<string, string>): {
  body: string | undefined;
  headers: Record<string, string> | undefined;
} {
  if (body === undefined || body === null) {
    return { body: undefined, headers };
  }
  if (typeof body === 'string') {
    return { body, headers };
  }
  if (typeof FormData !== 'undefined' && body instanceof FormData) {
    const encoded = formDataToUrlEncoded(body);
    const h = { ...headers };
    if (!hasHeader(h, 'Content-Type')) {
      h['Content-Type'] = 'application/x-www-form-urlencoded';
    }
    return { body: encoded, headers: h };
  }
  if (isPlainObject(body)) {
    const json = JSON.stringify(body);
    const h = { ...headers };
    if (!hasHeader(h, 'Content-Type')) {
      h['Content-Type'] = 'application/json';
    }
    return { body: json, headers: h };
  }
  return { body: JSON.stringify(body), headers };
}

/**
 * 清洗请求头：移除空 key/value
 */
function cleanHeaders(headers?: Record<string, string>): Record<string, string> | undefined {
  if (!headers) return undefined;
  const cleaned: Record<string, string> = {};
  for (const key in headers) {
    if (Object.prototype.hasOwnProperty.call(headers, key)) {
      const k = key.trim();
      const v = headers[key].trim();
      if (k && v) cleaned[k] = v;
    }
  }
  return cleaned;
}

/**
 * 将 HeadersInit 统一转为 Record
 */
function headersToRecord(headers: HeadersInit | undefined): Record<string, string> | undefined {
  if (!headers) return undefined;
  if (headers instanceof Headers) {
    const result: Record<string, string> = {};
    headers.forEach((v, k) => { result[k] = v; });
    return result;
  }
  if (Array.isArray(headers)) {
    const result: Record<string, string> = {};
    for (const [k, v] of headers) result[k] = v;
    return result;
  }
  return headers as Record<string, string>;
}

// ─── 流式支持 ───────────────────────────────────────────

/** 任务 ID 计数器 */
let _taskCounter = 0;

/** 生成唯一任务 ID */
function generateTaskId(): number {
  return Date.now() * 1000 + (++_taskCounter % 1000);
}

/**
 * 等待响应头到达（流式模式）
 */
function waitForResponseStart(
  taskId: number,
  timeout: number = 30000
): Promise<{ statusCode: number; statusText: string; headers: Record<string, string> }> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      unsubStart();
      unsubErr();
      reject(new Error(`HTTP response timeout for taskId ${taskId}`));
    }, timeout);

    const unsubStart = events.on(`http.response-start.${taskId}`, (data: any) => {
      clearTimeout(timer);
      unsubStart();
      unsubErr();
      resolve({
        statusCode: data.statusCode,
        statusText: data.statusText || '',
        headers: data.headers || {},
      });
    });

    const unsubErr = events.on(`http.error.${taskId}`, (data: any) => {
      clearTimeout(timer);
      unsubStart();
      unsubErr();
      reject(new Error(data.error));
    });
  });
}

// ─── HTTP API 实现 ──────────────────────────────────────

export const http: HttpAPI = {
  async fetch(input: string | URL, init?: HttpInit): Promise<Response> {
    const url = typeof input === 'string' ? input : input.toString();
    const isStream = init?.stream === true;

    // ── 非流式路径 ──
    if (!isStream) {
      const { body: resolvedBody, headers: resolvedHeaders } = resolveBody(
        init?.body,
        headersToRecord(init?.headers),
      );

      const raw = await vokexCall('http.request', {
        url,
        method: init?.method,
        headers: cleanHeaders(resolvedHeaders),
        body: resolvedBody,
        timeout: init?.timeout,
      });

      return new Response(new TextEncoder().encode(raw.body), {
        status: raw.statusCode,
        statusText: raw.statusText,
        headers: raw.headers,
      });
    }

    // ── 流式路径 ──
    const taskId = generateTaskId();

    const { body: resolvedBody, headers: resolvedHeaders } = resolveBody(
      init?.body,
      headersToRecord(init?.headers),
    );

    let chunkUnsub: (() => void) | null = null;
    let endUnsub: (() => void) | null = null;
    let errorUnsub: (() => void) | null = null;

    const stream = new ReadableStream<Uint8Array>({
      start(controller) {
        const encoder = new TextEncoder();

        chunkUnsub = events.on(`http.chunk.${taskId}`, (data: { data: string }) => {
          controller.enqueue(encoder.encode(data.data));
        });

        endUnsub = events.on(`http.end.${taskId}`, () => {
          controller.close();
          cleanup();
        });

        errorUnsub = events.on(`http.error.${taskId}`, (data: { error: string }) => {
          controller.error(new Error(data.error));
          cleanup();
        });
      },
      cancel() {
        cleanup();
      },
    });

    function cleanup() {
      chunkUnsub?.();
      endUnsub?.();
      errorUnsub?.();
      chunkUnsub = endUnsub = errorUnsub = null;
    }

    vokexCall('http.request', {
      url,
      method: init?.method,
      headers: cleanHeaders(resolvedHeaders),
      body: resolvedBody,
      timeout: init?.timeout,
      stream: true,
      taskId,
    });

    const responseData = await waitForResponseStart(taskId, (init?.timeout ?? 30) * 1000);

    return new Response(stream, {
      status: responseData.statusCode,
      statusText: responseData.statusText,
      headers: responseData.headers,
    });
  },

  get(url, init?) {
    return this.fetch(url, { ...init, method: 'GET' });
  },

  post(url, body?, init?) {
    return this.fetch(url, { ...init, method: 'POST', body });
  },

  put(url, body?, init?) {
    return this.fetch(url, { ...init, method: 'PUT', body });
  },

  delete(url, init?) {
    return this.fetch(url, { ...init, method: 'DELETE' });
  },
};
