/**
 * Google Fonts APIから必要な文字だけサブセットしたフォントを取得する。
 * `text`パラメータで動的サブセットし、Cache APIでキャッシュする。
 */
export async function loadNotoSansJP(
  text: string,
  weight: number = 400
): Promise<ArrayBuffer> {
  const params = new URLSearchParams({
    family: `Noto Sans JP:wght@${weight}`,
    text,
  });
  const cssUrl = `https://fonts.googleapis.com/css2?${params.toString()}`;

  // CSS取得（TTF形式をリクエスト）
  const cssResponse = await cachedFetch(cssUrl, {
    "User-Agent":
      "Mozilla/5.0 (Macintosh; U; Intel Mac OS X 10_6_8; de-at) AppleWebKit/533.21.1 (KHTML, like Gecko) Version/5.0.5 Safari/533.21.1",
  });

  const css = await cssResponse.text();
  const fontUrl = css.match(
    /src: url\((.+?)\) format\('(opentype|truetype)'\)/
  )?.[1];
  if (!fontUrl) {
    throw new Error("Could not find font URL in CSS response");
  }

  // フォントバイナリ取得
  const fontResponse = await cachedFetch(fontUrl);
  return fontResponse.arrayBuffer();
}

/** Cache API経由でfetchし、未キャッシュ時はs-maxage=86400でキャッシュに保存する */
async function cachedFetch(
  url: string,
  headers?: Record<string, string>
): Promise<Response> {
  const cache = caches.default;

  const cached = await cache.match(url);
  if (cached) return cached;

  const response = await fetch(url, headers ? { headers } : undefined);
  const toCache = new Response(response.body, response);
  toCache.headers.set("Cache-Control", "s-maxage=86400");
  await cache.put(url, toCache);

  return (await cache.match(url)) as Response;
}
