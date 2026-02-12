export interface VNode {
  type: string;
  props: {
    style?: Record<string, unknown>;
    children?: string | VNode | (string | VNode)[];
    [prop: string]: unknown;
  };
}

const OG_WIDTH = 1200;
const OG_HEIGHT = 630;
const BACKGROUND = "linear-gradient(135deg, #fafafa 0%, #f0f0f0 100%)";
const FONT_FAMILY = "Noto Sans JP, sans-serif";

/** satori向けdiv VNodeを生成する（display: "flex"は自動付与） */
function div(
  style: Record<string, unknown>,
  children: VNode["props"]["children"]
): VNode {
  return {
    type: "div",
    props: { style: { display: "flex", ...style }, children },
  };
}

/** 日記エントリー用OG画像テンプレート（1200x630） */
export function ogImageTemplate(date: string, contentPreview: string): VNode {
  return div(
    {
      flexDirection: "column",
      width: OG_WIDTH,
      height: OG_HEIGHT,
      background: BACKGROUND,
      fontFamily: FONT_FAMILY,
      padding: 60,
    },
    [
      div(
        {
          alignItems: "center",
          marginBottom: 30,
          fontSize: 28,
          color: "#888",
          letterSpacing: "0.05em",
        },
        "誰かが書く日記"
      ),
      div(
        {
          marginBottom: 24,
          fontSize: 48,
          fontWeight: 700,
          color: "#2c3e50",
        },
        `${date}の日記`
      ),
      div(
        {
          flex: 1,
          overflow: "hidden",
          fontSize: 32,
          lineHeight: 1.6,
          color: "#333",
        },
        contentPreview
      ),
      div(
        {
          justifyContent: "flex-end",
          marginTop: "auto",
          fontSize: 22,
          color: "#3498db",
        },
        "darekagakaku.day"
      ),
    ]
  );
}

/** デフォルトOG画像テンプレート（1200x630） */
export function ogImageDefaultTemplate(): VNode {
  return div(
    {
      flexDirection: "column",
      alignItems: "center",
      justifyContent: "center",
      width: OG_WIDTH,
      height: OG_HEIGHT,
      background: BACKGROUND,
      fontFamily: FONT_FAMILY,
    },
    [
      div(
        {
          fontSize: 64,
          fontWeight: 700,
          color: "#2c3e50",
          marginBottom: 20,
        },
        "誰かが書く日記"
      ),
      div(
        {
          fontSize: 28,
          color: "#888",
        },
        "自分が書かなければおそらく誰かが書く日記"
      ),
      div(
        {
          fontSize: 22,
          color: "#3498db",
          marginTop: 40,
        },
        "darekagakaku.day"
      ),
    ]
  );
}
