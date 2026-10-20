// Tooling adapter: parse normative YAML without losing alpha, fallback-font
// lists or unitless line heights in @google/design.md 0.4's Tailwind emitter.
// The official CLI still validates the source and supplies the DTCG export.
const text = await Bun.file("DESIGN.md").text();
const match = text.match(/^---\n([\s\S]*?)\n---/);
if (!match) throw Error("DESIGN.md frontmatter missing");
const source = Bun.YAML.parse(match[1]) as {
  colors: Record<string, string>;
  typography: Record<string, Record<string, string | number>>;
  spacing: Record<string, string | number>;
  rounded: Record<string, string>;
};
// design.md has no color modes: `light-<role>` is each role's system-light value.
const roles = Object.entries(source.colors).filter(([name]) => !name.startsWith("light-"));
const light = roles.map(([name]) => {
  const value = source.colors[`light-${name}`];
  if (value === undefined) throw Error(`DESIGN.md color ${name} needs light-${name}`);
  return [name, value];
});
if (roles.length + light.length !== Object.keys(source.colors).length) throw Error("Every light-* color must name a base role");
const lines = ["/* Generated from DESIGN.md. Do not edit. */", ":root {", "  color-scheme: dark light;"];
const css = (value: string | number) => String(value).replace(/\{colors\.([^}]+)\}/g, "var(--color-$1)");
for (const [name, value] of roles) lines.push(`  --color-${name}: ${css(value)};`);
const keys = { fontFamily: "font", fontSize: "text", fontWeight: "font-weight", lineHeight: "leading", letterSpacing: "tracking" };
for (const [name, style] of Object.entries(source.typography)) {
  for (const [prop, prefix] of Object.entries(keys)) {
    if (style[prop] !== undefined) lines.push(`  --${prefix}-${name}: ${style[prop]};`);
  }
}
for (const [name, value] of Object.entries(source.spacing)) lines.push(`  --spacing-${name}: ${value};`);
for (const [name, value] of Object.entries(source.rounded)) lines.push(`  --radius-${name}: ${value};`);
lines.push("}", "@media (prefers-color-scheme: light) {", "  :root {");
for (const [name, value] of light) lines.push(`    --color-${name}: ${css(value)};`);
lines.push("  }", "}", "");
await Bun.write("design/tokens.css", lines.join("\n"));

const dtcg = await Bun.file(".cache/official-tokens.json").json();
for (const [name, style] of Object.entries(source.typography)) {
  // 0.4 drops numeric lineHeight; restore it from the same normative source.
  // Preserve the fallback stack as a font-family array, not one fictitious font.
  const value = dtcg.typography[name].$value;
  if (style.lineHeight !== undefined) value.lineHeight = style.lineHeight;
  value.fontFamily = String(style.fontFamily).split(",").map(s => s.trim().replace(/^"(.*)"$/, "$1"));
}
await Bun.write("design/tokens.json", JSON.stringify(dtcg, null, 2) + "\n");
