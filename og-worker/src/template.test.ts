import { describe, it, expect } from "vitest";
import { ogImageTemplate, ogImageDefaultTemplate, VNode } from "./template";

/** VNodeツリーからすべてのテキストchildrenを再帰的に収集する */
function collectTexts(node: VNode | string): string[] {
  if (typeof node === "string") return [node];
  const { children } = node.props;
  if (children == null) return [];
  const childArray = Array.isArray(children) ? children : [children];
  return childArray.flatMap(collectTexts);
}

/** VNodeツリー内に指定テキストが含まれるか */
function containsText(node: VNode, text: string): boolean {
  return collectTexts(node).some((t) => t.includes(text));
}

describe("ogImageTemplate", () => {
  it("日付と内容プレビューを含む", () => {
    const vnode = ogImageTemplate("2025-01-15", "今日はいい天気だった");
    expect(containsText(vnode, "2025-01-15の日記")).toBe(true);
    expect(containsText(vnode, "今日はいい天気だった")).toBe(true);
  });

  it("サイト名とURLを含む", () => {
    const vnode = ogImageTemplate("2025-01-15", "テスト");
    expect(containsText(vnode, "誰かが書く日記")).toBe(true);
    expect(containsText(vnode, "darekagakaku.day")).toBe(true);
  });

  it("1200x630のサイズ指定がある", () => {
    const vnode = ogImageTemplate("2025-01-15", "テスト");
    expect(vnode.props.style?.width).toBe(1200);
    expect(vnode.props.style?.height).toBe(630);
  });

  it("ルート要素がdivでdisplay:flexを持つ", () => {
    const vnode = ogImageTemplate("2025-01-15", "テスト");
    expect(vnode.type).toBe("div");
    expect(vnode.props.style?.display).toBe("flex");
  });
});

describe("ogImageDefaultTemplate", () => {
  it("サイト名を含む", () => {
    const vnode = ogImageDefaultTemplate();
    expect(containsText(vnode, "誰かが書く日記")).toBe(true);
  });

  it("サブタイトルを含む", () => {
    const vnode = ogImageDefaultTemplate();
    expect(containsText(vnode, "自分が書かなければおそらく誰かが書く日記")).toBe(
      true
    );
  });

  it("URLを含む", () => {
    const vnode = ogImageDefaultTemplate();
    expect(containsText(vnode, "darekagakaku.day")).toBe(true);
  });

  it("1200x630のサイズ指定がある", () => {
    const vnode = ogImageDefaultTemplate();
    expect(vnode.props.style?.width).toBe(1200);
    expect(vnode.props.style?.height).toBe(630);
  });
});
