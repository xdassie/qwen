// rust-diagram-generator.js
// Run with: node rust-diagram-generator.js
// Outputs: rust_structure.svg

const fs = require('fs');

function generateSVG() {
  const width = 1400;
  const height = 950;
  
  return `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" 
     viewBox="0 0 ${width} ${height}" width="${width}" height="${height}"
     font-family="Inter, system-ui, -apple-system, sans-serif">
  
  <defs>
    <style>
      .module { fill: #1a1a2e; stroke: #16213e; stroke-width: 2; }
      .module-header { fill: #0f3460; }
      .func { fill: #16213e; stroke: #e94560; stroke-width: 2; }
      .struct { fill: #16213e; stroke: #0f3460; stroke-width: 2; }
      .test { fill: #16213e; stroke: #533483; stroke-width: 2; }
      .label { font-size: 12px; fill: #e0e0e0; }
      .title { font-size: 20px; font-weight: 600; fill: #16213e; }
      .subtitle { font-size: 14px; font-weight: 500; fill: #16213e; }
      .arrow { stroke: #16213e; stroke-width: 2; fill: none; marker-end: url(#arrowhead); }
      .scene-box { fill: #f0f4f8; stroke: #3d5a80; stroke-width: 2; }
      .object { fill: #fff; stroke: #16213e; stroke-width: 2; }
      .component { fill: #e94560; stroke: #1a1a2e; stroke-width: 2; }
      .text-line { fill: #a0a0a0; font-size: 11px; }
      .highlight { fill: #66fcf1; opacity: 0.2; }
      .label-header { font-weight: 600; }
      .label-title { font-weight: 500; }
      .object-label { font-size: 14px; font-weight: 600; }
      .object-sub { font-size: 10px; fill: #666; }
      .legend-box { fill: #f0f4f8; stroke: #ccc; }
    </style>
    
    <marker id="arrowhead" markerWidth="12" markerHeight="8" refX="11" refY="4" orient="auto">
      <polygon points="0 0, 12 4, 0 8" fill="#16213e" />
    </marker>
    
    <filter id="shadow" x="-20%" y="-20%" width="140%" height="140%">
      <feDropShadow dx="0" dy="2" stdDeviation="2" flood-color="#000000" flood-opacity="0.15"/>
    </filter>
  </defs>
  
  <!-- Background -->
  <rect x="0" y="0" width="${width}" height="${height}" fill="#ffffff"/>
  
  <!-- Title -->
  <text x="${width/2}" y="35" text-anchor="middle" class="title">Rust Bevy Graphics Module</text>
  <text x="${width/2}" y="55" text-anchor="middle" class="subtitle">Module Structure &amp; Scene Layout</text>
  
  <!-- === MODULE HIERARCHY === -->
  
  <!-- Main module -->
  <rect x="40" y="70" width="440" height="190" class="module" rx="10" ry="10" filter="url(#shadow)"/>
  <rect x="40" y="70" width="440" height="30" class="module-header" rx="10" ry="10"/>
  <text x="55" y="90" class="label label-header">src/</text>
  
  <rect x="60" y="95" width="400" height="70" class="module" rx="5" ry="5"/>
  <text x="80" y="120" class="label label-header">main.rs</text>
  <text x="60" y="155" class="text-line">mod graphics;</text>
  <text x="60" y="170" class="text-line">fn main() {</text>
  <text x="80" y="185" class="text-line">  graphics::run(true);</text>
  <text x="60" y="200" class="text-line">}</text>
  
  <!-- Graphics module -->
  <rect x="500" y="70" width="440" height="190" class="module" rx="10" ry="10" filter="url(#shadow)"/>
  <rect x="500" y="70" width="440" height="30" class="module-header" rx="10" ry="10"/>
  <text x="515" y="90" class="label label-header">src/graphics/</text>
  
  <rect x="520" y="95" width="400" height="70" class="module" rx="5" ry="5"/>
  <text x="540" y="120" class="label label-header">mod.rs</text>
  <text x="520" y="155" class="text-line">pub mod bevy;</text>
  <text x="520" y="170" class="text-line">pub use bevy::run;</text>
  
  <!-- Bevy module -->
  <rect x="960" y="70" width="300" height="530" class="module" rx="10" ry="10" filter="url(#shadow)"/>
  <rect x="960" y="70" width="300" height="30" class="module-header" rx="10" ry="10"/>
  <text x="975" y="90" class="label label-header">bevy.rs</text>
  
  <text x="970" y="115" class="text-line label-title" style="font-weight:500">use bevy::prelude::*;</text>
  <text x="970" y="130" class="text-line label-title" style="font-weight:500">use std::sync::atomic::*;</text>
  
  <rect x="970" y="145" width="260" height="20" class="module" rx="5" ry="5"/>
  <text x="970" y="158" class="text-line">static TEST_MODE: AtomicBool</text>
  
  <rect x="970" y="175" width="260" height="90" class="test" rx="5" ry="5"/>
  <text x="970" y="188" class="text-line label-title">#[cfg(test)] mod tests</text>
  <text x="970" y="203" class="text-line" style="font-size:10px">  test_test_mode_functions</text>
  <text x="970" y="218" class="text-line" style="font-size:10px">  test_run_with_test_mode</text>
  
  <rect x="970" y="285" width="260" height="150" class="func" rx="5" ry="5"/>
  <text x="970" y="298" class="text-line" style="font-weight:600; font-size:13px">pub fn run(test_mode: bool)</text>
  <text x="970" y="313" class="text-line">  if test_mode { enable_test_mode() }</text>
  <text x="970" y="328" class="text-line">  App::new()</text>
  <text x="970" y="343" class="text-line">    .add_plugins(DefaultPlugins)</text>
  <text x="970" y="358" class="text-line">    .add_systems(Startup, setup)</text>
  <text x="970" y="373" class="text-line">    .add_systems(Update, spin_cube)</text>
  <text x="970" y="388" class="text-line">    .add_systems(Update, check_clicks)</text>
  <text x="970" y="403" class="text-line">    .add_systems(Update, handle_clicks)</text>
  <text x="970" y="418" class="text-line">    .run();</text>
  
  <rect x="970" y="450" width="260" height="140" class="struct" rx="5" ry="5"/>
  <text x="970" y="463" class="text-line" style="font-weight:600; font-size:13px">struct Clicked</text>
  <text x="970" y="478" class="text-line">  entity: Vec3,</text>
  <text x="970" y="493" class="text-line">  position: Vec3,</text>
  <text x="970" y="508" class="text-line">  min_pos: Vec3,</text>
  <text x="970" y="523" class="text-line">  max_pos: Vec3,</text>
  <text x="970" y="538" class="text-line">  center: Vec3,</text>
  <text x="970" y="553" class="text-line">  size: Vec3,</text>
  <text x="970" y="568" class="text-line">  color_handle: Handle&lt;Material&gt;</text>
  
  <rect x="970" y="605" width="260" height="160" class="func" rx="5" ry="5"/>
  <text x="970" y="618" class="text-line" style="font-weight:600; font-size:13px">System Functions</text>
  <text x="970" y="633" class="text-line">  setup() - spawn objects</text>
  <text x="970" y="648" class="text-line">  spin_cube() - rotation</text>
  <text x="970" y="663" class="text-line">  check_clicks() - raycast</text>
  <text x="970" y="678" class="text-line">  handle_clicks() - update</text>
  
  <!-- Connection arrows between modules -->
  <line x1="440" y1="155" x2="500" y2="155" class="arrow"/>
  <line x1="900" y1="155" x2="960" y2="155" class="arrow"/>
  
  <!-- === 3D SCENE LAYOUT === -->
  
  <rect x="120" y="280" width="900" height="540" class="scene-box" rx="12" ry="12" filter="url(#shadow)"/>
  <text x="150" y="305" class="label" style="font-weight:600; font-size:16px">3D Scene Objects</text>
  <text x="150" y="320" class="label" style="font-size:11px; fill:#666">Objects spawned in setup() function</text>
  
  <!-- Camera -->
  <rect x="1000" y="350" width="140" height="45" class="component" rx="8" ry="8"/>
  <text x="1015" y="368" class="label" style="font-weight:600">Camera3d</text>
  <text x="1015" y="380" class="label" style="font-size:10px; fill:#666">pos: (0, 0, 5)</text>
  <text x="1015" y="395" class="label" style="font-size:10px; fill:#666">looking_at: (0, 0, 0)</text>
  
  <!-- Cube - center -->
  <rect x="520" y="350" width="100" height="100" class="object" rx="12" ry="12" filter="url(#shadow)"/>
  <rect x="520" y="350" width="100" height="100" class="highlight"/>
  <text x="570" y="342" class="label object-label">Cube</text>
  <text x="570" y="365" class="label object-sub">blue</text>
  <text x="570" y="378" class="label object-sub">pos: (0, 0, 0)</text>
  <text x="570" y="390" class="label object-sub">⬅ spinning ⬅</text>
  
  <!-- Sphere - left -->
  <circle cx="440" cy="430" r="35" class="object" fill="#ffebee" filter="url(#shadow)"/>
  <text x="440" y="435" class="label object-label">Sphere</text>
  <text x="440" y="452" class="label object-sub">red</text>
  <text x="440" y="465" class="label object-sub">pos: (-2, 0, 1)</text>
  
  <!-- Cylinder - right -->
  <rect x="660" y="400" width="70" height="90" class="object" rx="5" ry="5" filter="url(#shadow)"/>
  <text x="700" y="425" class="label object-label">Cylinder</text>
  <text x="700" y="438" class="label object-sub">green</text>
  <text x="700" y="451" class="label object-sub">pos: (2, 0, 0)</text>
  
  <!-- Torus - bottom -->
  <ellipse cx="570" cy="510" rx="40" ry="25" class="object" fill="#fffde7" filter="url(#shadow)"/>
  <text x="570" y="497" class="label object-label">Torus</text>
  <text x="570" y="512" class="label object-sub">yellow</text>
  <text x="570" y="525" class="label object-sub">pos: (0, 1.5, 0)</text>
  
  <!-- Plane - bottom -->
  <rect x="200" y="540" width="220" height="15" class="object" rx="2" ry="2" fill="#fffde7" stroke="#f9a825" stroke-width="2"/>
  <text x="310" y="547" class="label" style="font-size:11px; fill:#fbc02d">Plane (mentioned in test)</text>
  <text x="310" y="560" class="label object-sub">pos: (0, -2, 0), yellow</text>
  
  <!-- Lights -->
  <rect x="140" y="350" width="180" height="35" class="component" rx="6" ry="6"/>
  <text x="230" y="368" class="label" style="font-weight:600">AmbientLight</text>
  <text x="230" y="380" class="label" style="font-size:10px; fill:#666">brightness: 500</text>
  
  <rect x="140" y="400" width="180" height="35" class="component" rx="6" ry="6"/>
  <text x="230" y="418" class="label" style="font-weight:600">DirectionalLight</text>
  <text x="230" y="430" class="label" style="font-size:10px; fill:#666">pos: (5, 5, 5)</text>
  
  <!-- === CLICK DETECTION SECTION === -->
  
  <rect x="120" y="600" width="900" height="180" class="scene-box" rx="12" ry="12" filter="url(#shadow)"/>
  <text x="150" y="625" class="label" style="font-weight:600; font-size:16px">Click Detection (Test Mode Only)</text>
  
  <rect x="140" y="635" width="520" height="85" fill="#e8f5e9" stroke="#66bb6a" stroke-width="2" rx="8" ry="8"/>
  <text x="150" y="650" class="label" style="font-weight:600">On mouse click:</text>
  <text x="150" y="665" class="text-line">1. Spawn Clicked component</text>
  <text x="150" y="680" class="text-line">2. Store bounds (min/max positions)</text>
  <text x="150" y="695" class="text-line">3. Print color &amp; position</text>
  
  <rect x="680" y="635" width="360" height="85" fill="#fff3e0" stroke="#fb8c00" stroke-width="2" rx="8" ry="8"/>
  <text x="690" y="650" class="label" style="font-weight:600">Clicked object bounds:</text>
  <text x="690" y="665" class="text-line">position: transform.translation</text>
  <text x="690" y="680" class="text-line">min_pos: position - (0.5, 0.5, 0.5)</text>
  <text x="690" y="695" class="text-line">max_pos: position + (0.5, 0.5, 0.5)</text>
  
  <!-- === LEGEND === -->
  
  <rect x="120" y="800" width="80" height="50" class="legend-box" rx="5" ry="5"/>
  <text x="135" y="815" class="label" style="font-weight:600; font-size:11px">Legend</text>
  
  <rect x="130" y="825" width="15" height="10" class="object"/>
  <text x="150" y="832" class="label" style="font-size:10px">3D Object</text>
  
  <rect x="180" y="825" width="15" height="10" class="component"/>
  <text x="200" y="832" class="label" style="font-size:10px">Component</text>
  
  <rect x="240" y="825" width="15" height="10" fill="#e8f5e9" stroke="#66bb6a" stroke-width="2"/>
  <text x="260" y="832" class="label" style="font-size:10px">Clicked</text>
  
  <rect x="320" y="825" width="15" height="10" fill="#fffde7" stroke="#f9a825" stroke-width="2"/>
  <text x="340" y="832" class="label" style="font-size:10px">Bounds</text>
  
</svg>`;
}

// Generate and save
const svg = generateSVG();
fs.writeFileSync('rust_structure.svg', svg);
console.log('✓ Generated rust_structure.svg');

// Also generate an interactive HTML version
function generateHTML() {
  const svg = generateSVG();
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Rust Bevy Module Structure</title>
  <style>
    body { 
      font-family: 'Inter', system-ui, sans-serif;
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      margin: 0;
      padding: 20px;
      min-height: 100vh;
    }
    .container {
      max-width: 1400px;
      margin: 0 auto;
      background: white;
      border-radius: 16px;
      box-shadow: 0 20px 60px rgba(0,0,0,0.3);
      overflow: hidden;
    }
    header {
      background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
      color: white;
      padding: 30px;
      text-align: center;
    }
    h1 { margin: 0; font-size: 28px; }
    .subtitle { opacity: 0.8; margin-top: 8px; }
    .content { padding: 0; }
    svg { display: block; margin: 0 auto; }
    .hover-box {
      cursor: pointer;
      transition: all 0.3s ease;
    }
    .hover-box:hover {
      transform: translateY(-3px);
      box-shadow: 0 10px 30px rgba(0,0,0,0.2);
    }
    .controls {
      padding: 20px 30px;
      background: #f8f9fa;
      border-bottom: 1px solid #dee2e6;
      display: flex;
      gap: 15px;
    }
    button {
      padding: 10px 20px;
      border: none;
      border-radius: 8px;
      background: #1a1a2e;
      color: white;
      cursor: pointer;
      font-size: 14px;
      transition: all 0.2s;
    }
    button:hover {
      background: #e94560;
      transform: scale(1.05);
    }
    #info-panel {
      position: fixed;
      top: 20px;
      right: 20px;
      background: white;
      padding: 15px 20px;
      border-radius: 12px;
      box-shadow: 0 10px 40px rgba(0,0,0,0.2);
      max-width: 300px;
      display: none;
      z-index: 1000;
    }
    #info-panel h3 {
      margin: 0 0 10px 0;
      color: #16213e;
      font-size: 14px;
    }
    #info-panel p {
      margin: 5px 0;
      font-size: 12px;
      color: #666;
    }
  </style>
</head>
<body>
  <div class="container">
    <header>
      <h1>Rust Bevy Graphics Module</h1>
      <div class="subtitle">Module Structure &amp; Scene Layout</div>
    </header>
    <div class="controls">
      <button onclick="zoomIn()">🔍 Zoom In</button>
      <button onclick="zoomOut()">🔍 Zoom Out</button>
      <button onclick="resetView()">↻ Reset</button>
    </div>
    <div class="content">
      ${svg}
    </div>
  </div>
  
  <div id="info-panel">
    <h3 id="info-title">Title</h3>
    <div id="info-details"></div>
  </div>
  
  <script>
    let currentScale = 1;
    let isDragging = false;
    let startX, startY;
    let viewBox = [120, 280, 900, 540];
    
    const svg = document.querySelector('svg');
    const sceneBox = document.querySelector('.scene-box');
    const infoPanel = document.getElementById('info-panel');
    const infoTitle = document.getElementById('info-title');
    const infoDetails = document.getElementById('info-details');
    
    function zoomIn() {
      currentScale = Math.min(currentScale * 1.2, 3);
      updateTransform();
    }
    
    function zoomOut() {
      currentScale = Math.max(currentScale / 1.2, 0.5);
      updateTransform();
    }
    
    function resetView() {
      currentScale = 1;
      updateTransform();
    }
    
    function updateTransform() {
      svg.style.transform = `scale(${currentScale})`;
      svg.style.transformOrigin = 'center center';
      svg.style.width = `${1400 * currentScale}px`;
      svg.style.height = `${950 * currentScale}px`;
    }
    
    // Mouse interactions
    sceneBox.addEventListener('mousemove', (e) => {
      const rect = sceneBox.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      
      // Show info panel when hovering over elements
      infoPanel.style.display = 'block';
      infoPanel.style.left = (e.clientX + 15) + 'px';
      infoPanel.style.top = (e.clientY + 15) + 'px';
    });
    
    sceneBox.addEventListener('mouseleave', () => {
      infoPanel.style.display = 'none';
    });
    
    // Object labels for hover info
    sceneBox.querySelectorAll('.object-label').forEach(label => {
      label.addEventListener('mouseenter', (e) => {
        const name = e.target.textContent.trim();
        infoTitle.textContent = name;
        infoDetails.innerHTML = '<p>Interactive diagram - hover over elements</p>';
      });
    });
    
    // Initialize
    updateTransform();
    
    // Responsive
    window.addEventListener('resize', () => {
      const container = document.querySelector('.content');
      const maxWidth = container.offsetWidth;
      svg.style.maxWidth = maxWidth + 'px';
    });
  </script>
</body>
</html>`;
}

const html = generateHTML();
fs.writeFileSync('rust_diagram.html', html);
console.log('✓ Generated rust_diagram.html (interactive version)');
