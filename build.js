// build.js - 修复别名错误版
const esbuild = require('esbuild');
const fs = require('fs');
const path = require('path');

// 确保assets目录存在
const assetsDir = path.join(__dirname, 'assets');
if (!fs.existsSync(assetsDir)) {
  fs.mkdirSync(assetsDir, { recursive: true });
}

// 构建配置
const buildConfig = {
  // 注意：如果你的 editor.js 放在 assets 文件夹下，这里要改为 ['assets/editor.js']
  // 如果它和 build.js 同级（根目录），则保持 ['editor.js']
  entryPoints: ['editor.js'], 
  bundle: true,
  outfile: 'assets/bundle_editor.js',
  format: 'esm',
  platform: 'browser',
  target: ['es2020'],
  external: [],
  loader: {
    '.js': 'js',
    '.css': 'css'
  },
  define: {
    'process.env.NODE_ENV': '"production"'
  },
  minify: true,
  sourcemap: true,
  treeShaking: true,
  legalComments: 'none'
  // 删除了引发报错的 alias 配置，因为相对路径在 esbuild 中会自动解析
};

// 构建函数
async function build() {
  console.log('🚀 开始构建编辑器...');
  
  try {
    // 执行构建
    const result = await esbuild.build(buildConfig);
    
    console.log('✅ 构建成功!');
    console.log(`📦 输出文件: ${buildConfig.outfile}`);
    console.log(`📏 文件大小: ${(fs.statSync(buildConfig.outfile).size / 1024).toFixed(2)} KB`);
    
    // 生成构建信息文件
    const buildInfo = {
      timestamp: new Date().toISOString(),
      version: require('./package.json').version,
      buildConfig: {
        entryPoints: buildConfig.entryPoints,
        format: buildConfig.format,
        platform: buildConfig.platform,
        target: buildConfig.target
      }
    };
    
    fs.writeFileSync(
      'assets/build-info.json',
      JSON.stringify(buildInfo, null, 2)
    );
    
    console.log('📝 构建信息已保存到 assets/build-info.json');
    
  } catch (error) {
    console.error('❌ 构建失败:', error.message);
    if (error.errors) {
      error.errors.forEach(err => console.error('   ', err.text));
    }
    process.exit(1);
  }
}

// 开发模式监听
async function watch() {
  console.log('👀 进入开发模式，监听文件变化...');
  
  try {
    const ctx = await esbuild.context(buildConfig);
    await ctx.watch();
    console.log('✅ 监听已启动，修改文件将自动重新构建...');
  } catch (error) {
    console.error('❌ 监听失败:', error);
    process.exit(1);
  }
}

// 根据命令行参数决定执行构建还是监听
if (process.argv.includes('--watch')) {
  watch();
} else {
  build();
}