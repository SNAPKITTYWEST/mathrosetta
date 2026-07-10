#!/usr/bin/env node
/**
 * SNAPKITTY DOCS GENERATOR
 * Continuously generates documentation from code
 * Run: node generate-docs.mjs
 * Or: node generate-docs.mjs --watch
 */

import { readdirSync, readFileSync, writeFileSync, statSync, existsSync, mkdirSync } from 'fs'
import { join, basename, extname } from 'path'
import { spawn } from 'child_process'

const CONFIG = {
  inputDirs: ['src', 'lib', 'proofs', 'docs'],
  outputDir: 'docs/generated',
  extensions: ['.rs', '.lean', '.py', '.apl', '.hs', '.mjs', '.js', '.pl', '.md'],
  ignore: ['node_modules', 'target', 'build', '.git'],
  watch: process.argv.includes('--watch'),
  interval: 5000 // 5 seconds
}

function log(msg, level = 'info') {
  const colors = { error: '\x1b[31m', warn: '\x1b[33m', info: '\x1b[36m', success: '\x1b[32m' }
  const color = colors[level] || ''
  const reset = '\x1b[0m'
  console.log(`[${new Date().toISOString()}] ${color}${level.toUpperCase()}: ${msg}${reset}`)
}

function isIgnored(dir) {
  return CONFIG.ignore.includes(basename(dir))
}

function isValidFile(file) {
  return CONFIG.extensions.includes(extname(file).toLowerCase())
}

function extractDocComment(content, ext) {
  const patterns = {
    '.rs': /\/\*\*(.*?)\*\/|\/\/\!\s*(.*)/gs,
    '.lean': /\/\-\!(.*?)$/gm,
    '.py': /"""(.*?)"""|#\s*(.*)/gs,
    '.apl': /\⍝\s*(.*)/g,
    '.hs': /\-\-\!(.*?)$/gm,
    '.mjs': /\/\*\*(.*?)\*\/|\/\/\s*(.*)/gs,
    '.js': /\/\*\*(.*?)\*\/|\/\/\s*(.*)/gs,
    '.pl': /#\s*(.*)/g,
    '.md': /(.*?)/gs
  }
  
  const pattern = patterns[ext] || /\/\*(.*?)\*\//gs
  const matches = [...content.matchAll(pattern)]
  
  return matches.map(m => {
    const text = (m[1] || m[2] || m[0] || '').trim()
    return text ? text.replace(/^\s*[\/*#⍝\-]+\s*/gm, '').trim() : null
  }).filter(Boolean).join('\n\n')
}

function parseFunction(content, ext) {
  // Extract function signatures
  const signatures = []
  
  if (['.rs', '.mjs', '.js'].includes(ext)) {
    const funcPattern = /fn\s+(\w+)\s*\(([^)]*)\)\s*[^{]*{/g
    for (const match of content.matchAll(funcPattern)) {
      signatures.push(`fn ${match[1]}(${match[2]})`)
    }
  }
  
  if (['.lean'].includes(ext)) {
    const funcPattern = /def\s+(\w+)\s*\(([^)]*)\)\s*:/g
    for (const match of content.matchAll(funcPattern)) {
      signatures.push(`def ${match[1]}(${match[2]})`)
    }
  }
  
  if (['.py'].includes(ext)) {
    const funcPattern = /def\s+(\w+)\s*\(([^)]*)\)\s*:/g
    for (const match of content.matchAll(funcPattern)) {
      signatures.push(`def ${match[1]}(${match[2]})`)
    }
  }
  
  return signatures
}

function generateFileDoc(filePath) {
  try {
    const content = readFileSync(filePath, 'utf8')
    const ext = extname(filePath)
    const relativePath = filePath.replace(process.cwd() + '/', '')
    const docComment = extractDocComment(content, ext)
    const functions = parseFunction(content, ext)
    
    let doc = `# ${relativePath}\n\n`
    
    if (docComment) {
      doc += `## Description\n\n${docComment}\n\n`
    }
    
    if (functions.length > 0) {
      doc += `## Functions/Definitions\n\n`
      doc += functions.map(f => `- \`${f}\``).join('\n')
      doc += '\n\n'
    }
    
    doc += `---\n\n`
    doc += `[Back to index](./index.md)\n`
    
    return doc
  } catch (e) {
    log(`Error processing ${filePath}: ${e.message}`, 'warn')
    return null
  }
}

function scanFiles(dir = '.') {
  const files = []
  
  try {
    const entries = readdirSync(dir)
    
    for (const entry of entries) {
      const fullPath = join(dir, entry)
      const stat = statSync(fullPath)
      
      if (isIgnored(fullPath)) continue
      
      if (stat.isDirectory()) {
        files.push(...scanFiles(fullPath))
      } else if (stat.isFile() && isValidFile(fullPath)) {
        files.push(fullPath)
      }
    }
  } catch (e) {
    log(`Error scanning ${dir}: ${e.message}`, 'warn')
  }
  
  return files
}

function generateIndex(files) {
  const grouped = {}
  
  for (const file of files) {
    const ext = extname(file)
    const dir = basename(join(file, '..'))
    
    if (!grouped[ext]) grouped[ext] = []
    if (!grouped[dir]) grouped[dir] = []
    
    const relative = file.replace(process.cwd() + '/', '')
    grouped[ext].push(relative)
    grouped[dir].push(relative)
  }
  
  let index = '# Documentation Index\n\n'
  index += 'Automatically generated from source code\n\n'
  index += `---\n\n`
  
  // By directory
  index += '## By Directory\n\n'
  for (const [dir, files] of Object.entries(grouped)) {
    if (dir.startsWith('.')) continue
    index += `\n### ${dir}/\n\n`
    for (const file of files.sort()) {
      const link = file.replace(/\\/g, '/').replace(/ /g, '%20')
      index += `- [${file}](${link}.md)\n`
    }
  }
  
  // By type
  index += '\n\n## By Type\n\n'
  for (const [ext, files] of Object.entries(grouped)) {
    if (!ext.startsWith('.')) continue
    const type = ext.substring(1).toUpperCase()
    index += `\n### ${type}\n\n`
    for (const file of files.sort()) {
      const link = file.replace(/\\/g, '/').replace(/ /g, '%20')
      index += `- [${file}](${link}.md)\n`
    }
  }
  
  return index
}

function generateAll() {
  log('Starting documentation generation', 'info')
  
  // Ensure output directory exists
  if (!existsSync(CONFIG.outputDir)) {
    mkdirSync(CONFIG.outputDir, { recursive: true })
  }
  
  // Scan files
  const files = scanFiles()
  log(`Found ${files.length} source files`, 'info')
  
  // Generate index
  const index = generateIndex(files)
  writeFileSync(join(CONFIG.outputDir, 'index.md'), index)
  log('Generated index.md', 'info')
  
  // Generate individual file docs
  for (const file of files) {
    const doc = generateFileDoc(file)
    if (doc) {
      const relativePath = file.replace(process.cwd() + '/', '')
      const outputPath = join(CONFIG.outputDir, relativePath + '.md')
      
      // Ensure directory exists
      const outputDir = join(CONFIG.outputDir, join(file, '..').replace(process.cwd() + '/', ''))
      if (!existsSync(outputDir)) {
        mkdirSync(outputDir, { recursive: true })
      }
      
      writeFileSync(outputPath, doc)
    }
  }
  
  log(`Generated documentation for ${files.length} files`, 'success')
  
  // Commit and push if in a git repo
  try {
    spawn('git', ['add', CONFIG.outputDir], { stdio: 'inherit' })
    spawn('git', ['commit', '-m', 'Update generated docs'], { stdio: 'inherit' })
    spawn('git', ['push'], { stdio: 'inherit' })
    log('Committed and pushed documentation', 'success')
  } catch (e) {
    log(`Could not commit docs: ${e.message}`, 'warn')
  }
}

function watch() {
  log('Watching for changes...', 'info')
  generateAll()
  
  setInterval(() => {
    generateAll()
  }, CONFIG.interval)
}

// Main
if (CONFIG.watch) {
  watch()
} else {
  generateAll()
}
