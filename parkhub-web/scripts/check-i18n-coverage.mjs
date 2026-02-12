#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';

const scriptDir = path.dirname(new URL(import.meta.url).pathname);
const webRoot = path.resolve(scriptDir, '..');
const i18nDir = path.join(webRoot, 'src', 'i18n');
const baselineFile = path.join(webRoot, 'scripts', 'i18n-coverage-baseline.json');
const updateBaseline = process.argv.includes('--update-baseline');

const files = fs
  .readdirSync(i18nDir)
  .filter((file) => /^(en|de|es|fr|zh|pt|ar|hi|ja|tr)\.ts$/.test(file))
  .sort();

function loadTranslation(filePath) {
  const source = fs.readFileSync(filePath, 'utf8');
  const transformed = source.replace(/export const \w+\s*=\s*/, 'module.exports = ');
  const context = { module: { exports: {} }, exports: {} };
  vm.runInNewContext(transformed, context, { filename: filePath });
  return context.module.exports;
}

function flattenKeys(obj, prefix = '', out = new Set()) {
  for (const [key, value] of Object.entries(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      flattenKeys(value, fullKey, out);
    } else {
      out.add(fullKey);
    }
  }
  return out;
}

const translations = new Map();
for (const file of files) {
  const locale = path.basename(file, '.ts');
  translations.set(locale, flattenKeys(loadTranslation(path.join(i18nDir, file))));
}

const enKeys = translations.get('en');
if (!enKeys || enKeys.size === 0) {
  console.error('❌ i18n check failed: en.ts missing or has no keys.');
  process.exit(1);
}

const report = {};
for (const [locale, keys] of translations.entries()) {
  if (locale === 'en') continue;
  report[locale] = {
    missing: [...enKeys].filter((key) => !keys.has(key)).sort(),
    extra: [...keys].filter((key) => !enKeys.has(key)).sort(),
  };
}

if (updateBaseline) {
  fs.writeFileSync(baselineFile, JSON.stringify(report, null, 2) + '\n');
  console.log(`✅ Updated baseline: ${path.relative(webRoot, baselineFile)}`);
  process.exit(0);
}

const baseline = JSON.parse(fs.readFileSync(baselineFile, 'utf8'));
let hasRegression = false;

for (const [locale, diff] of Object.entries(report)) {
  const baselineLocale = baseline[locale] ?? { missing: [], extra: [] };
  const baselineMissing = new Set(baselineLocale.missing);
  const baselineExtra = new Set(baselineLocale.extra);

  const newMissing = diff.missing.filter((key) => !baselineMissing.has(key));
  const newExtra = diff.extra.filter((key) => !baselineExtra.has(key));

  if (newMissing.length > 0 || newExtra.length > 0) {
    hasRegression = true;
    console.error(`\n❌ ${locale}: new i18n key coverage regressions detected`);
    if (newMissing.length) {
      console.error(`  New missing keys (${newMissing.length}):`);
      newMissing.slice(0, 20).forEach((key) => console.error(`    - ${key}`));
    }
    if (newExtra.length) {
      console.error(`  New extra keys (${newExtra.length}):`);
      newExtra.slice(0, 20).forEach((key) => console.error(`    + ${key}`));
    }
  }
}

if (hasRegression) {
  console.error('\n❌ i18n coverage regression check failed.');
  process.exit(1);
}

const unresolved = Object.values(report).reduce(
  (acc, item) => ({ missing: acc.missing + item.missing.length, extra: acc.extra + item.extra.length }),
  { missing: 0, extra: 0 },
);

console.log(
  `✅ i18n regression check passed (${enKeys.size} en keys, unresolved debt: ${unresolved.missing} missing / ${unresolved.extra} extra).`,
);
