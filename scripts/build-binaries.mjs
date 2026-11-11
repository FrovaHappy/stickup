#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { existsSync, readdirSync, symlinkSync, copyFileSync } from 'node:fs';
import { join } from 'node:path';
import process from 'node:process';

function getPnpmExecutable() {
  return process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm';
}

function run(cmd, args = []) {
  const isWindows = process.platform === 'win32';
  const executable = (cmd === 'pnpm' || cmd === 'npm' || cmd === 'npx') && isWindows
    ? `${cmd}.cmd`
    : cmd;

  console.log(`\n> ${cmd} ${args.join(' ')}`);

  // On Windows, running batch scripts (.cmd) requires shell: true
  const result = spawnSync(executable, args, {
    stdio: 'inherit',
    shell: isWindows,
  });

  if (result.status !== 0) {
    console.error(`\n❌ Error: El comando "${cmd} ${args.join(' ')}" finalizó con código ${result.status}`);
    process.exit(result.status ?? 1);
  }
}

function prepareWindowsXwin() {
  if (process.platform === 'win32') {
    // Si la compilación se ejecuta nativamente en Windows, no se necesita cargo-xwin ni symlinks
    return;
  }

  console.log('\n[Windows Cross-Compile] Preparando caché de xwin y asegurando enlaces de compatibilidad...');
  try {
    const isWindows = process.platform === 'win32';
    const cargoBin = isWindows ? 'cargo.exe' : 'cargo';
    spawnSync(cargoBin, ['xwin', 'cache', 'xwin'], { stdio: 'inherit', shell: isWindows });
  } catch (err) {
    console.warn('Advertencia: No se pudo ejecutar "cargo xwin cache xwin":', err.message);
  }

  const candidateDirs = [
    process.env.XWIN_CACHE_DIR,
    join(process.cwd(), 'src-tauri/target/.xwin-cache'),
    join(process.cwd(), 'target/.xwin-cache'),
    join(process.cwd(), '.xwin-cache'),
    process.env.HOME ? join(process.env.HOME, '.cache/xwin') : null,
    process.env.HOME ? join(process.env.HOME, '.xwin') : null,
  ].filter(Boolean);

  for (const dir of candidateDirs) {
    fixCasingInDirectory(dir);
  }
}

function fixCasingInDirectory(baseDir) {
  if (!existsSync(baseDir)) return;

  const targets = [
    { lower: 'directml.lib', target: 'DirectML.lib' },
    { lower: 'pathcch.lib', target: 'PathCch.lib' },
  ];

  function walk(dir) {
    try {
      const entries = readdirSync(dir, { withFileTypes: true });
      for (const entry of entries) {
        const fullPath = join(dir, entry.name);
        if (entry.isDirectory()) {
          walk(fullPath);
        } else if (entry.isFile()) {
          for (const { lower, target } of targets) {
            if (entry.name === lower) {
              const targetPath = join(dir, target);
              if (!existsSync(targetPath)) {
                try {
                  symlinkSync(lower, targetPath);
                  console.log(`[xwin] Enlace simbólico creado: ${targetPath} -> ${lower}`);
                } catch {
                  try {
                    copyFileSync(fullPath, targetPath);
                    console.log(`[xwin] Archivo copiado: ${targetPath}`);
                  } catch (e) {
                    console.warn(`[xwin] No se pudo crear ${targetPath}:`, e.message);
                  }
                }
              }
            }
          }
        }
      }
    } catch {
      // Ignorar errores de acceso
    }
  }

  walk(baseDir);
}

// 1. Parsear plataformas de los argumentos
const rawArgs = process.argv.slice(2);
const requested = rawArgs
  .flatMap((arg) => arg.split(/[,\s]+/))
  .map((p) => p.toLowerCase().trim())
  .filter(Boolean);

let platforms = [];
if (requested.includes('all')) {
  platforms = ['linux', 'windows'];
} else {
  for (const p of requested) {
    if (['linux', 'lin'].includes(p) && !platforms.includes('linux')) {
      platforms.push('linux');
    } else if (['windows', 'win', 'win64'].includes(p) && !platforms.includes('windows')) {
      platforms.push('windows');
    }
  }
}

if (platforms.length === 0) {
  console.log(`
Uso: pnpm build:binaries <plataformas...>

Plataformas soportadas:
  linux     Compila paquetes para Linux (.deb, .AppImage y binario)
  windows   Compila instalador para Windows (.exe NSIS y binario)
  all       Compila ambas plataformas

Ejemplos:
  pnpm build:binaries linux
  pnpm build:binaries windows
  pnpm build:binaries linux windows
  pnpm build:binaries all
`);
  process.exit(1);
}

console.log(`\n========================================`);
console.log(`🛠️  StickUp - Compilación unificada`);
console.log(`📦 Plataformas destino: ${platforms.join(', ')}`);
console.log(`========================================`);

// 2. Compilar el frontend una sola vez
console.log('\n[Paso 1] Compilando frontend (Vite)...');
run('pnpm', ['run', 'build']);

// 3. Compilar cada plataforma con Tauri reutilizando el frontend precompilado
const noBeforeBuild = JSON.stringify({ build: { beforeBuildCommand: '' } });

for (let i = 0; i < platforms.length; i++) {
  const platform = platforms[i];
  const stepLabel = `[Paso ${i + 2}/${platforms.length + 1}]`;

  if (platform === 'linux') {
    console.log(`\n${stepLabel} Compilando paquetes para Linux (deb, appimage)...`);
    run('pnpm', [
      'tauri',
      'build',
      '--bundles',
      'deb,appimage',
      '--config',
      noBeforeBuild,
    ]);
  } else if (platform === 'windows') {
    console.log(`\n${stepLabel} Compilando paquetes para Windows (nsis)...`);
    prepareWindowsXwin();

    const isHostWindows = process.platform === 'win32';
    const tauriArgs = ['tauri', 'build', '--bundles', 'nsis'];

    if (!isHostWindows) {
      tauriArgs.push('--runner', 'cargo-xwin', '--target', 'x86_64-pc-windows-msvc');
    }

    tauriArgs.push('--config', noBeforeBuild);
    run('pnpm', tauriArgs);
  }
}

console.log(`\n✨ ¡Compilación completada exitosamente para: ${platforms.join(', ')}!`);

