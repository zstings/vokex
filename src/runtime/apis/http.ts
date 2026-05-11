import vokexCall from "./vokexCall";

/**
 * RequestOptions HTTP 请求选项（与 fetch API 一致）
 */
export interface RequestOptions {
  /** 请求方法 */
  method?: string;
  /** 请求头 */
  headers?: Record<string, string>;
  /** 请求体（支持 string / 纯对象 / FormData） */
  body?: any;
  /** 超时时间（秒） */
  timeout?: number;
}

/**
 * 原始响应数据（Rust 后端返回的 JSON 结构）
 */
interface RawResponse {
  statusCode: number;
  statusText: string;
  headers: Record<string, string>;
  body: string;
  ok: boolean;
}

// ─── 工具函数 ────────────────────────────────────────────

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
 * 将 FormData 转换为 URL-encoded 字符串（a=1&b=2 格式）
 * 仅处理键值对，暂不支持文件上传
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
  // 无 body，直接返回
  if (body === undefined || body === null) {
    return { body: undefined, headers };
  }

  // 字符串，原样传递
  if (typeof body === 'string') {
    return { body, headers };
  }

  // FormData → URL-encoded
  if (typeof FormData !== 'undefined' && body instanceof FormData) {
    const encoded = formDataToUrlEncoded(body);
    const h = { ...headers };
    if (!hasHeader(h, 'Content-Type')) {
      h['Content-Type'] = 'application/x-www-form-urlencoded';
    }
    return { body: encoded, headers: h };
  }

  // 纯对象 → JSON
  if (isPlainObject(body)) {
    const json = JSON.stringify(body);
    const h = { ...headers };
    if (!hasHeader(h, 'Content-Type')) {
      h['Content-Type'] = 'application/json';
    }
    return { body: json, headers: h };
  }

  // 其他类型（数组等），尝试 JSON.stringify
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
      if (k && v) {
        cleaned[k] = v;
      }
    }
  }
  return cleaned;
}

// ─── 不区分大小写的 Headers 包装 ────────────────────────

/**
 * 不区分大小写的 Headers 包装
 *
 * 内部存储统一使用小写 key，所有读写操作自动转小写，
 * 确保 headers.get('Content-Type') 与 headers.get('content-type') 等价。
 */
export class VokexHeaders {
  private _map: Map<string, string>;

  constructor(raw?: Record<string, string>) {
    this._map = new Map();
    if (raw) {
      for (const key in raw) {
        if (Object.prototype.hasOwnProperty.call(raw, key)) {
          this._map.set(key.toLowerCase(), raw[key]);
        }
      }
    }
  }

  /** 获取指定 header 的值（不区分大小写） */
  get(name: string): string | null {
    return this._map.get(name.toLowerCase()) ?? null;
  }

  /** 检查是否存在指定 header（不区分大小写） */
  has(name: string): boolean {
    return this._map.has(name.toLowerCase());
  }

  /** 遍历所有 header */
  forEach(callback: (value: string, key: string) => void): void {
    this._map.forEach(callback);
  }

  /** 返回所有 header 的迭代器 [key, value][] */
  entries(): IterableIterator<[string, string]> {
    return this._map.entries();
  }

  /** 返回所有 header key */
  keys(): IterableIterator<string> {
    return this._map.keys();
  }

  /** 返回所有 header value */
  values(): IterableIterator<string> {
    return this._map.values();
  }

  /** 转为普通对象（key 全小写） */
  toObject(): Record<string, string> {
    const result: Record<string, string> = {};
    this._map.forEach((v, k) => {
      result[k] = v;
    });
    return result;
  }
}

// ─── VokexResponse ──────────────────────────────────────

/**
 * VokexResponse — 模拟浏览器 fetch Response 的响应对象
 */
export class VokexResponse {
  /** 响应状态码 */
  readonly status: number;
  /** 响应状态文本 */
  readonly statusText: string;
  /** 响应头（不区分大小写访问） */
  readonly headers: VokexHeaders;
  /** 响应是否成功（状态码 200-299） */
  readonly ok: boolean;
  /** 原始响应体字符串 */
  private _body: string;
  /** body 是否已被消费 */
  private _bodyUsed: boolean = false;

  constructor(raw: RawResponse) {
    this.status = raw.statusCode;
    this.statusText = raw.statusText;
    this.headers = new VokexHeaders(raw.headers);
    this.ok = raw.ok;
    this._body = raw.body;
  }

  /** body 是否已被消费 */
  get bodyUsed(): boolean {
    return this._bodyUsed;
  }

  /** 以文本形式读取响应体 */
  async text(): Promise<string> {
    this._bodyUsed = true;
    return this._body;
  }

  /** 以 JSON 形式读取响应体 */
  async json<T = any>(): Promise<T> {
    this._bodyUsed = true;
    return JSON.parse(this._body);
  }

  /** 克隆响应（允许重复读取 body） */
  clone(): VokexResponse {
    return new VokexResponse({
      statusCode: this.status,
      statusText: this.statusText,
      headers: this.headers.toObject(),
      body: this._body,
      ok: this.ok,
    });
  }
}

// ─── HTTP API ───────────────────────────────────────────

/**
 * HTTP API 接口
 */
export interface HttpAPI {
  /** 发送 HTTP 请求 */
  request: (url: string, options?: RequestOptions) => Promise<VokexResponse>;
  /** GET 请求 */
  get: (url: string, options?: RequestOptions) => Promise<VokexResponse>;
  /** POST 请求 */
  post: (url: string, body?: any, options?: RequestOptions) => Promise<VokexResponse>;
  /** PUT 请求 */
  put: (url: string, body?: any, options?: RequestOptions) => Promise<VokexResponse>;
  /** DELETE 请求 */
  delete: (url: string, options?: RequestOptions) => Promise<VokexResponse>;
}

/**
 * 核心请求逻辑：统一处理 body 智能解析 + header 清洗
 */
async function doRequest(url: string, options?: RequestOptions): Promise<VokexResponse> {
  const { body: resolvedBody, headers: resolvedHeaders } = resolveBody(
    options?.body,
    options?.headers,
  );

  const raw: RawResponse = await vokexCall('http.request', {
    url,
    method: options?.method,
    headers: cleanHeaders(resolvedHeaders),
    body: resolvedBody,
    timeout: options?.timeout,
  });

  return new VokexResponse(raw);
}

/**
 * HTTP 请求 API（后端代理，绕过 CORS）
 */
export const http: HttpAPI = {
  /**
   * 发送 HTTP 请求
   * @param url 请求地址
   * @param options 请求选项
   */
  request(url: string, options?: RequestOptions): Promise<VokexResponse> {
    return doRequest(url, options);
  },

  /**
   * GET 请求
   * @param url 请求地址
   * @param options 请求选项
   */
  get(url: string, options?: RequestOptions): Promise<VokexResponse> {
    return doRequest(url, { ...options, method: 'GET' });
  },

  /**
   * POST 请求
   * @param url 请求地址
   * @param body 请求体（string / 纯对象 / FormData）
   * @param options 请求选项
   */
  post(url: string, body?: any, options?: RequestOptions): Promise<VokexResponse> {
    return doRequest(url, { ...options, method: 'POST', body });
  },

  /**
   * PUT 请求
   * @param url 请求地址
   * @param body 请求体（string / 纯对象 / FormData）
   * @param options 请求选项
   */
  put(url: string, body?: any, options?: RequestOptions): Promise<VokexResponse> {
    return doRequest(url, { ...options, method: 'PUT', body });
  },

  /**
   * DELETE 请求
   * @param url 请求地址
   * @param options 请求选项
   */
  delete(url: string, options?: RequestOptions): Promise<VokexResponse> {
    return doRequest(url, { ...options, method: 'DELETE' });
  },
};
