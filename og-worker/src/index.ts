import { satori } from "@cf-wasm/satori/workerd";
import { Resvg } from "@cf-wasm/resvg/workerd";
import { loadNotoSansJP } from "./font";
import { ogImageTemplate, ogImageDefaultTemplate } from "./template";

export default {
  async fetch(
    request: Request,
    _env: unknown,
    ctx: ExecutionContext
  ): Promise<Response> {
    const url = new URL(request.url);
    const path = url.pathname;

    const dateMatch = path.match(/^\/og\/(\d{4}-\d{2}-\d{2})\.png$/);
    const isDefault = path === "/og/default.png";

    if (!dateMatch && !isDefault) {
      return new Response("Not found", { status: 404 });
    }

    // キャッシュ確認
    const cache = caches.default;
    const cacheKey = new Request(url.toString(), { method: "GET" });
    const cachedResponse = await cache.match(cacheKey);
    if (cachedResponse) {
      return cachedResponse;
    }

    let element;
    let textForFont: string;

    if (isDefault) {
      element = ogImageDefaultTemplate();
      textForFont =
        "誰かが書く日記自分が書かなければおそらく誰かが書く日記darekagakaku.day";
    } else {
      const date = dateMatch![1];
      // メインWorkerがD1から取得した内容をクエリパラメータで渡してくれる
      const contentPreview =
        url.searchParams.get("content") || "この日の日記";
      element = ogImageTemplate(date, contentPreview);
      textForFont = `誰かが書く日記${date}の日記${contentPreview}darekagakaku.day`;
    }

    // フォント用に使用文字を重複排除
    const uniqueChars = [...new Set(textForFont)].join("");

    const [fontData, fontDataBold] = await Promise.all([
      loadNotoSansJP(uniqueChars, 400),
      loadNotoSansJP(uniqueChars, 700),
    ]);

    // VNode → SVG (satori)
    const svg = await satori(element, {
      width: 1200,
      height: 630,
      fonts: [
        {
          name: "Noto Sans JP",
          data: fontData,
          weight: 400,
          style: "normal",
        },
        {
          name: "Noto Sans JP",
          data: fontDataBold,
          weight: 700,
          style: "normal",
        },
      ],
    });

    // SVG → PNG (resvg)
    const resvg = await Resvg.async(svg, {
      fitTo: { mode: "width", value: 1200 },
    });
    const pngBuffer = resvg.render().asPng();

    const response = new Response(pngBuffer, {
      headers: {
        "Content-Type": "image/png",
        "Cache-Control": "public, s-maxage=3600, max-age=3600",
      },
    });

    ctx.waitUntil(cache.put(cacheKey, response.clone()));

    return response;
  },
};
