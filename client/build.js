const esbuild = require('esbuild');
const fs = require('fs');
const path = require('path');

const isWatch = process.argv.includes('--watch');

const buildOptions = {
  entryPoints: ['src/index.ts'],
  bundle: true,
  minify: !isWatch,
  sourcemap: isWatch,
  target: ['es2020'],
  format: 'iife',
  globalName: 'CommentWidget',
  outfile: 'dist/comment-widget.js',
  loader: {
    '.css': 'text',
  },
};

async function build() {
  if (isWatch) {
    const ctx = await esbuild.context(buildOptions);
    await ctx.watch();
    console.log('👀 监听中... 修改 src/ 后自动构建');
  } else {
    await esbuild.build(buildOptions);

    // 同时生成 ESM 版本
    await esbuild.build({
      ...buildOptions,
      format: 'esm',
      globalName: undefined,
      outfile: 'dist/comment-widget.esm.js',
    });

    console.log('✅ 构建完成: dist/comment-widget.js + dist/comment-widget.esm.js');

    // 生成 CSS
    const cssContent = fs.readFileSync(
      path.join(__dirname, 'src', 'style.css'),
      'utf-8'
    );
    fs.writeFileSync(
      path.join(__dirname, 'dist', 'comment-widget.css'),
      cssContent
    );
    console.log('✅ CSS: dist/comment-widget.css');
  }
}

build().catch((err) => {
  console.error('构建失败:', err);
  process.exit(1);
});
