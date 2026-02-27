import Foundation

extension ChatWebView {
    static func buildHTML(title: String) -> String {
        let escapedTitle = title.replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "\"", with: "\\\"")

        // Get theme CSS variables from ThemeManager
        let themeCSS = ThemeManager.shared.generateCSSVariables(for: ConfigManager.shared.theme.name)

        return """
        <!DOCTYPE html>
        <html>
        <head>
        <meta charset="utf-8">
        <script src="https://cdn.jsdelivr.net/npm/d3@7/dist/d3.min.js"></script>
        <script src="https://cdn.jsdelivr.net/npm/marked@15/marked.min.js"></script>
        <style id="jarvis-theme-vars">
        \(themeCSS)
        </style>
        <style>
            * { margin: 0; padding: 0; box-sizing: border-box; }
            body {
                background: var(--color-panel-bg);
                color: var(--color-primary);
                font-family: var(--font-family);
                font-size: var(--font-size);
                line-height: var(--line-height);
                display: flex;
                flex-direction: column;
                height: 100vh;
                overflow: hidden;
                border: 1px solid var(--color-border);
                transition: border-color 0.2s ease;
            }
            body.focused {
                border-color: var(--color-border-focused);
                box-shadow: inset 0 0 12px color-mix(in srgb, var(--color-primary) 8%, transparent);
            }
            #title-bar {
                padding: 14px 20px 8px;
                font-size: var(--font-title-size);
                font-weight: bold;
                color: var(--color-primary);
                text-shadow: 0 0 8px color-mix(in srgb, var(--color-primary) 35%, transparent);
                border-bottom: 1px solid var(--color-border);
                flex-shrink: 0;
                display: flex;
                justify-content: space-between;
                align-items: center;
            }
            #title-bar .close-btn {
                cursor: pointer;
                opacity: 0.3;
                font-size: 14px;
                line-height: 1;
                padding: 2px 6px;
                border-radius: 3px;
                transition: opacity 0.15s ease, background 0.15s ease;
            }
            #title-bar .close-btn:hover {
                opacity: 0.8;
                background: color-mix(in srgb, var(--color-primary) 15%, transparent);
            }
            #messages {
                flex: 1;
                overflow-y: auto;
                padding: 10px 20px;
            }
            #messages::-webkit-scrollbar { width: 3px; }
            #messages::-webkit-scrollbar-track { background: transparent; }
            #messages::-webkit-scrollbar-thumb { background: color-mix(in srgb, var(--color-primary) 15%, transparent); border-radius: 2px; }

            .msg { margin-bottom: 6px; word-wrap: break-word; }
            .msg.gemini { color: var(--color-text); }
            .msg.gemini h1, .msg.gemini h2, .msg.gemini h3 {
                font-size: 14px; margin: 10px 0 4px; color: var(--color-text);
                text-shadow: 0 0 6px color-mix(in srgb, var(--color-text) 15%, transparent);
            }
            .msg.gemini h1 { font-size: 15px; }
            .msg.gemini p { margin: 4px 0; }
            .msg.gemini ul, .msg.gemini ol { margin: 4px 0 4px 20px; }
            .msg.gemini li { margin: 2px 0; }
            .msg.gemini strong { color: var(--color-text); filter: brightness(1.1); }
            .msg.gemini code {
                background: color-mix(in srgb, var(--color-primary) 8%, transparent);
                padding: 1px 4px; border-radius: 2px;
                font-size: 12px;
            }
            .msg.gemini pre {
                background: color-mix(in srgb, var(--color-primary) 5%, transparent);
                padding: 8px; border-radius: 3px;
                margin: 6px 0; overflow-x: auto;
            }
            .msg.gemini pre code { background: none; padding: 0; }
            .msg.user {
                color: var(--color-user-text);
                padding: 4px 0;
            }
            .msg.user::before { content: '> '; opacity: 0.4; }
            .msg.user-image {
                margin: 8px 0 4px;
                padding: 0;
            }
            .msg.user-image img {
                max-width: 100%;
                max-height: 300px;
                border-radius: 4px;
                border: 1px solid var(--color-border);
                display: block;
            }
            .msg.iframe-container {
                margin: 8px 0;
                padding: 0;
            }
            .msg.iframe-container iframe {
                width: 100%;
                border: 1px solid var(--color-border);
                border-radius: 4px;
                background: #111;
                display: block;
            }

            /* Tool activity — Claude Code style */
            .msg.tool-activity {
                font-size: 12px;
                padding: 4px 10px 3px 12px;
                margin: 8px 0 0;
                border-left: 2px solid;
                white-space: pre-wrap;
                font-weight: 600;
            }
            .msg.tool-activity .tool-label {
                opacity: 0.55;
                font-weight: normal;
                font-size: 11px;
                margin-right: 4px;
            }
            .msg.tool_read  { color: var(--color-tool-read); border-color: color-mix(in srgb, var(--color-tool-read) 40%, transparent); }
            .msg.tool_edit  { color: var(--color-tool-edit); border-color: color-mix(in srgb, var(--color-tool-edit) 40%, transparent); }
            .msg.tool_write { color: var(--color-tool-write); border-color: color-mix(in srgb, var(--color-tool-write) 40%, transparent); }
            .msg.tool_run   { color: var(--color-tool-run); border-color: color-mix(in srgb, var(--color-tool-run) 40%, transparent); }
            .msg.tool_search{ color: var(--color-tool-search); border-color: color-mix(in srgb, var(--color-tool-search) 40%, transparent); }
            .msg.tool_list  { color: var(--color-primary); border-color: color-mix(in srgb, var(--color-primary) 35%, transparent); }
            .msg.tool_data  { color: color-mix(in srgb, var(--color-primary) 85%, cyan); border-color: color-mix(in srgb, var(--color-primary) 35%, transparent); }
            .msg.tool_tool  { color: #ffc832; border-color: rgba(255, 200, 50, 0.35); }
            @keyframes subagent-pulse {
                0%, 100% { opacity: 1; border-color: rgba(255, 200, 50, 0.35); }
                50% { opacity: 0.65; border-color: rgba(255, 200, 50, 0.85); }
            }
            .msg.tool_tool.running {
                animation: subagent-pulse 2s ease-in-out infinite;
            }
            .msg.tool_tool .elapsed {
                font-weight: normal;
                font-size: 10px;
                opacity: 0.45;
                margin-left: 8px;
            }
            .msg.tool_tool .current-op {
                font-weight: normal;
                font-size: 11px;
                opacity: 0.6;
                margin-left: 6px;
                font-style: italic;
            }
            .msg.tool_result {
                color: rgba(180, 190, 200, 0.5);
                font-size: 11px;
                padding: 2px 10px 4px 12px;
                border-left: 2px solid rgba(180, 190, 200, 0.12);
                margin: 0 0 6px;
                white-space: pre-wrap;
                max-height: 180px;
                overflow-y: auto;
            }
            .msg.subagent_result {
                color: rgba(140, 160, 180, 0.45);
                font-size: 10px;
                padding: 1px 10px 3px 20px;
                border-left: 2px solid rgba(100, 140, 180, 0.10);
                margin: 0 0 4px;
                white-space: pre-wrap;
                max-height: 140px;
                overflow-y: auto;
            }
            .msg.approval {
                color: rgba(255, 160, 50, 0.95);
                font-size: 13px;
                padding: 8px 12px;
                border: 1px solid rgba(255, 160, 50, 0.35);
                border-left: 3px solid rgba(255, 160, 50, 0.7);
                border-radius: 3px;
                margin: 8px 0;
                background: rgba(255, 160, 50, 0.06);
            }
            .msg.approval code {
                background: rgba(255, 160, 50, 0.12);
                padding: 2px 6px;
                border-radius: 2px;
                font-size: 12px;
                color: rgba(255, 200, 100, 1);
            }
            .msg.approval strong { color: rgba(255, 200, 100, 1); }

            .chart-container {
                margin: 10px 0;
                padding: 14px;
                background: color-mix(in srgb, var(--color-primary) 3%, transparent);
                border: 1px solid color-mix(in srgb, var(--color-primary) 10%, transparent);
                border-radius: 4px;
            }
            .chart-container svg text { fill: var(--color-primary); font-family: var(--font-family); font-size: 11px; }
            .chart-container svg .bar { fill: color-mix(in srgb, var(--color-primary) 55%, transparent); }
            .chart-container svg .bar:hover { fill: color-mix(in srgb, var(--color-primary) 80%, transparent); }
            .chart-container svg .line-path { fill: none; stroke: var(--color-primary); stroke-width: 2; }
            .chart-container svg .dot { fill: var(--color-primary); }
            .chart-container svg .axis line, .chart-container svg .axis path { stroke: color-mix(in srgb, var(--color-primary) 20%, transparent); }
            .chart-title { font-size: 13px; font-weight: bold; margin-bottom: 6px; color: var(--color-primary); }

            #input-bar {
                display: flex;
                padding: 6px 20px 10px;
                border-top: 1px solid var(--color-border);
                flex-shrink: 0;
            }
            #input-bar textarea {
                flex: 1;
                background: color-mix(in srgb, var(--color-primary) 5%, transparent);
                border: 1px solid var(--color-border);
                border-radius: 3px;
                color: var(--color-primary);
                font-family: var(--font-family);
                font-size: var(--font-size);
                padding: 7px 10px;
                outline: none;
                resize: none;
                overflow: hidden;
                min-height: 32px;
                max-height: 300px;
                line-height: 1.4;
            }
            #input-bar textarea::placeholder { color: color-mix(in srgb, var(--color-primary) 20%, transparent); }
            #input-bar textarea:focus { border-color: color-mix(in srgb, var(--color-primary) 40%, transparent); }
            #status-bar {
                padding: 4px 20px 0;
                font-size: 10px;
                color: color-mix(in srgb, var(--color-primary) 35%, transparent);
                flex-shrink: 0;
                font-family: var(--font-family);
            }
            #image-preview {
                display: none;
                padding: 8px 20px;
                border-top: 1px solid var(--color-border);
                flex-shrink: 0;
                position: relative;
            }
            #image-preview img {
                max-height: 200px;
                max-width: 100%;
                border-radius: 4px;
                border: 1px solid color-mix(in srgb, var(--color-primary) 20%, transparent);
                display: block;
            }
            #image-preview .close-btn {
                position: absolute;
                top: 4px;
                right: 24px;
                color: color-mix(in srgb, var(--color-primary) 50%, transparent);
                cursor: pointer;
                font-size: 16px;
                line-height: 1;
            }
            #image-preview .close-btn:hover {
                color: var(--color-primary);
            }
            #chat-overlay {
                position: fixed;
                top: 8px;
                right: 8px;
                max-width: 280px;
                font-size: 11px;
                line-height: 1.4;
                color: color-mix(in srgb, var(--color-primary) 40%, transparent);
                font-family: var(--font-family);
                white-space: pre-wrap;
                pointer-events: none;
                z-index: 100;
                text-align: right;
            }
        </style>
        </head>
        <body>
            <div id="chat-overlay"></div>
            <div id="title-bar"><span>[ \(escapedTitle) ]</span><span class="close-btn" onclick="closePanel()">&#x2715;</span></div>
            <div id="messages"></div>
            <div id="image-preview"></div>
            <div id="input-bar">
                <textarea id="chat-input" rows="1" placeholder="Type a question..." autocomplete="off"></textarea>
            </div>
            <div id="status-bar"></div>
        <script>
        // Configure marked for minimal output
        marked.setOptions({ breaks: true, gfm: true });

        const messages = document.getElementById('messages');
        const chatInput = document.getElementById('chat-input');
        let currentSpeaker = null;
        let currentEl = null;
        let geminiBuffer = '';  // accumulates full gemini response for markdown re-render
        let chartCounter = 0;

        function isNearBottom() {
            const threshold = 80;
            return messages.scrollHeight - messages.scrollTop - messages.clientHeight < threshold;
        }
        function scrollIfNear() {
            if (isNearBottom()) messages.scrollTop = messages.scrollHeight;
        }

        function appendChunk(speaker, text) {
            if (speaker === 'user') {
                // User messages: plain text, new element
                currentSpeaker = null;
                currentEl = null;
                geminiBuffer = '';
                const el = document.createElement('div');
                el.className = 'msg user';
                el.textContent = text;
                messages.appendChild(el);
                messages.scrollTop = messages.scrollHeight;
                return;
            }

            if (speaker === 'subagent_op') {
                // Update the live subagent row's current-operation text in-place
                const running = messages.querySelector('.tool_tool.running');
                if (running) {
                    const opSpan = running.querySelector('.current-op');
                    if (opSpan) opSpan.textContent = '\\u25b8 ' + text;
                    const prev = parseInt(running.dataset.subagentOpCount || '0') + 1;
                    running.dataset.subagentOpCount = String(prev);
                }
                return;
            }

            if (speaker === 'subagent_done') {
                currentSpeaker = null;
                currentEl = null;
                geminiBuffer = '';
                const running = messages.querySelector('.tool_tool.running');
                if (running) {
                    running.classList.remove('running');
                    const tid = running.dataset.timerId;
                    if (tid) clearInterval(parseInt(tid));
                    const opSpan = running.querySelector('.current-op');
                    const opCount = parseInt(text) || parseInt(running.dataset.subagentOpCount || '0');
                    if (opSpan) opSpan.textContent = opCount > 0 ? '(' + opCount + ' ops)' : '';
                }
                return;
            }

            if (speaker.startsWith('tool_') && speaker !== 'tool_result') {
                // Tool start — break gemini text flow so next text creates a new div below
                currentSpeaker = null;
                currentEl = null;
                geminiBuffer = '';
                const el = document.createElement('div');
                el.className = 'msg tool-activity ' + speaker;
                // Subagent: structured spans for live updates
                if (speaker === 'tool_tool' && text.startsWith('Subagent:')) {
                    const existing = messages.querySelector('.tool_tool.running');
                    if (existing) {
                        const descSpan = existing.querySelector('.subagent-desc');
                        if (descSpan && descSpan.textContent === text) return;
                    }
                    el.classList.add('running');
                    el.dataset.subagentOpCount = '0';
                    const desc = document.createElement('span');
                    desc.className = 'subagent-desc';
                    desc.textContent = text;
                    el.appendChild(desc);
                    const opSpan = document.createElement('span');
                    opSpan.className = 'current-op';
                    el.appendChild(opSpan);
                    const elapsed = document.createElement('span');
                    elapsed.className = 'elapsed';
                    elapsed.textContent = ' 0s';
                    el.appendChild(elapsed);
                    const start = Date.now();
                    const tid = setInterval(() => {
                        const s = Math.round((Date.now() - start) / 1000);
                        elapsed.textContent = s < 60 ? ' ' + s + 's' : ' ' + Math.floor(s/60) + 'm ' + (s%60) + 's';
                    }, 1000);
                    el.dataset.timerId = tid;
                } else {
                    el.textContent = text;
                }
                messages.appendChild(el);
                scrollIfNear();
                return;
            }

            if (speaker === 'subagent_result') {
                // Subagent internal tool result — indented, dimmer
                currentSpeaker = null;
                currentEl = null;
                geminiBuffer = '';
                const el = document.createElement('div');
                el.className = 'msg subagent_result';
                el.textContent = text;
                messages.appendChild(el);
                scrollIfNear();
                return;
            }

            if (speaker === 'tool_result') {
                // Tool result — dimmed output preview
                currentSpeaker = null;
                currentEl = null;
                geminiBuffer = '';
                const el = document.createElement('div');
                el.className = 'msg tool_result';
                el.textContent = text;
                messages.appendChild(el);
                scrollIfNear();
                return;
            }

            if (speaker === 'approval') {
                // Command approval request: render with markdown for code/bold
                currentSpeaker = null;
                currentEl = null;
                geminiBuffer = '';
                const el = document.createElement('div');
                el.className = 'msg approval';
                el.innerHTML = marked.parse(text);
                messages.appendChild(el);
                scrollIfNear();
                return;
            }

            // Gemini: accumulate and re-render as markdown
            geminiBuffer += text;

            // Check for complete chart blocks
            const chartRegex = /```chart\\n([\\s\\S]*?)\\n```/g;
            let hasChart = chartRegex.test(geminiBuffer);

            if (speaker !== currentSpeaker || !currentEl) {
                currentEl = document.createElement('div');
                currentEl.className = 'msg gemini';
                messages.appendChild(currentEl);
                currentSpeaker = speaker;
            }

            // Split buffer into text segments and chart blocks
            const parts = geminiBuffer.split(/(```chart\\n[\\s\\S]*?\\n```)/g);
            currentEl.innerHTML = '';

            for (const part of parts) {
                const chartMatch = part.match(/^```chart\\n([\\s\\S]*?)\\n```$/);
                if (chartMatch) {
                    try {
                        const config = JSON.parse(chartMatch[1]);
                        const container = document.createElement('div');
                        container.className = 'chart-container';
                        container.id = 'chart-' + (chartCounter++);
                        if (config.title) {
                            const t = document.createElement('div');
                            t.className = 'chart-title';
                            t.textContent = config.title;
                            container.appendChild(t);
                        }
                        const svgBox = document.createElement('div');
                        container.appendChild(svgBox);
                        currentEl.appendChild(container);
                        buildChart(svgBox, config);
                    } catch(e) {
                        const errEl = document.createElement('div');
                        errEl.textContent = '[chart error: ' + e.message + ']';
                        errEl.style.color = '#ff6666';
                        currentEl.appendChild(errEl);
                    }
                } else if (part.trim()) {
                    // Check if buffer ends mid-chart block (incomplete)
                    if (part.includes('```chart') && !part.includes('```chart\\n')) {
                        // Might be incomplete — render as text for now
                        const span = document.createElement('span');
                        span.innerHTML = marked.parse(part);
                        currentEl.appendChild(span);
                    } else {
                        const span = document.createElement('span');
                        span.innerHTML = marked.parse(part);
                        currentEl.appendChild(span);
                    }
                }
            }

            scrollIfNear();
        }

        function appendImage(dataUrl) {
            currentSpeaker = null;
            currentEl = null;
            geminiBuffer = '';
            const el = document.createElement('div');
            el.className = 'msg user-image';
            const img = document.createElement('img');
            img.src = dataUrl;
            el.appendChild(img);
            messages.appendChild(el);
            messages.scrollTop = messages.scrollHeight;
        }

        function appendIframe(url, height) {
            currentSpeaker = null;
            currentEl = null;
            geminiBuffer = '';
            const el = document.createElement('div');
            el.className = 'msg iframe-container';
            const iframe = document.createElement('iframe');
            iframe.src = url;
            iframe.style.height = height + 'px';
            iframe.setAttribute('sandbox', 'allow-scripts allow-same-origin allow-popups');
            el.appendChild(iframe);
            messages.appendChild(el);
            messages.scrollTop = messages.scrollHeight;
        }

        function appendIframeSrcdoc(base64, height) {
            currentSpeaker = null;
            currentEl = null;
            geminiBuffer = '';
            const el = document.createElement('div');
            el.className = 'msg iframe-container';
            const iframe = document.createElement('iframe');
            iframe.srcdoc = atob(base64);
            iframe.style.height = height + 'px';
            iframe.setAttribute('sandbox', 'allow-scripts allow-same-origin allow-popups');
            el.appendChild(iframe);
            messages.appendChild(el);
            messages.scrollTop = messages.scrollHeight;
        }

        function buildChart(container, config) {
            const w = 460, h = 180, m = {top: 15, right: 15, bottom: 35, left: 45};
            const iw = w - m.left - m.right, ih = h - m.top - m.bottom;

            const svg = d3.select(container).append('svg')
                .attr('width', w).attr('height', h)
                .append('g').attr('transform', `translate(${m.left},${m.top})`);

            const labels = config.labels || [];
            const values = config.values || [];

            if (config.type === 'bar') {
                const x = d3.scaleBand().domain(labels).range([0, iw]).padding(0.3);
                const y = d3.scaleLinear().domain([0, d3.max(values) * 1.1]).range([ih, 0]);
                svg.append('g').attr('class','axis').attr('transform',`translate(0,${ih})`).call(d3.axisBottom(x));
                svg.append('g').attr('class','axis').call(d3.axisLeft(y).ticks(5));
                svg.selectAll('.bar').data(values).join('rect')
                    .attr('class','bar').attr('x',(d,i)=>x(labels[i])).attr('y',d=>y(d))
                    .attr('width',x.bandwidth()).attr('height',d=>ih-y(d));
            } else if (config.type === 'line') {
                const x = d3.scalePoint().domain(labels).range([0, iw]);
                const y = d3.scaleLinear().domain([0, d3.max(values) * 1.1]).range([ih, 0]);
                svg.append('g').attr('class','axis').attr('transform',`translate(0,${ih})`).call(d3.axisBottom(x));
                svg.append('g').attr('class','axis').call(d3.axisLeft(y).ticks(5));
                const line = d3.line().x((d,i)=>x(labels[i])).y(d=>y(d));
                svg.append('path').datum(values).attr('class','line-path').attr('d',line);
                svg.selectAll('.dot').data(values).join('circle')
                    .attr('class','dot').attr('cx',(d,i)=>x(labels[i])).attr('cy',d=>y(d)).attr('r',3);
            } else if (config.type === 'pie') {
                const radius = Math.min(iw, ih) / 2;
                const g = svg.attr('transform',`translate(${m.left + iw/2},${m.top + ih/2})`);
                const colors = labels.map((_, i) => `hsl(${190 + i * 30}, 75%, ${50 + i * 5}%)`);
                const color = d3.scaleOrdinal().domain(labels).range(colors);
                const pie = d3.pie().value((d,i) => values[i]);
                const arc = d3.arc().innerRadius(0).outerRadius(radius);
                g.selectAll('path').data(pie(labels)).join('path')
                    .attr('d', arc).attr('fill', d => color(d.data)).attr('stroke','rgba(0,0,0,0.3)');
                g.selectAll('text').data(pie(labels)).join('text')
                    .attr('transform', d => `translate(${arc.centroid(d)})`)
                    .attr('text-anchor','middle').attr('font-size','10px')
                    .text(d => d.data);
            }
        }

        const IMAGE_PATH_RE = /\\/\\S+\\.(?:png|jpg|jpeg|gif|webp|bmp|tiff|heic)/i;
        let currentPreviewPath = null;

        function checkForImagePath() {
            const match = chatInput.value.match(IMAGE_PATH_RE);
            if (match && match[0] !== currentPreviewPath) {
                currentPreviewPath = match[0];
                window.webkit.messageHandlers.chatInput.postMessage('__preview_image__' + match[0]);
            } else if (!match && currentPreviewPath) {
                clearImagePreview();
            }
        }

        function showImagePreview(dataUrl) {
            const preview = document.getElementById('image-preview');
            preview.innerHTML = '<img src="' + dataUrl + '"><span class="close-btn" onclick="clearImagePreview()">\\u00d7</span>';
            preview.style.display = 'block';
        }

        function clearImagePreview() {
            const preview = document.getElementById('image-preview');
            preview.innerHTML = '';
            preview.style.display = 'none';
            currentPreviewPath = null;
        }

        function autoGrow() {
            chatInput.style.height = 'auto';
            chatInput.style.overflow = 'hidden';
            const sh = chatInput.scrollHeight;
            chatInput.style.height = Math.min(sh, 300) + 'px';
            chatInput.style.overflow = sh > 300 ? 'auto' : 'hidden';
        }

        chatInput.addEventListener('input', function() {
            autoGrow();
            checkForImagePath();
        });

        let lastEscapeJS = 0;
        chatInput.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                const now = Date.now();
                if (now - lastEscapeJS < 400) {
                    // Double escape → clear input
                    lastEscapeJS = 0;
                    chatInput.value = '';
                    if (typeof autoGrow === 'function') autoGrow();
                    if (typeof clearImagePreview === 'function') clearImagePreview();
                } else {
                    // Single escape → nothing
                    lastEscapeJS = now;
                }
                return;
            }
            if (e.key === 'Enter' && !e.shiftKey && chatInput.value.trim()) {
                e.preventDefault();
                const text = chatInput.value.trim();
                window.webkit.messageHandlers.chatInput.postMessage(text);
                chatInput.value = '';
                autoGrow();
                clearImagePreview();
            }
        });

        function setInputText(text) {
            if (chatInput.value.trim()) {
                chatInput.value += ' ' + text;
            } else {
                chatInput.value = text;
            }
            autoGrow();
            chatInput.focus();
        }

        function setChatOverlay(text) {
            document.getElementById('chat-overlay').textContent = text;
        }

        function setStatus(text) {
            document.getElementById('status-bar').textContent = text;
        }

        function setFocused(isFocused) {
            if (isFocused) {
                document.body.classList.add('focused');
                chatInput.focus();
            } else {
                document.body.classList.remove('focused');
            }
        }

        document.addEventListener('mousedown', () => {
            window.webkit.messageHandlers.chatInput.postMessage('__focus__');
        });

        var _iframeKeyForwarder = null;

        function showFullscreenIframe(base64) {
            document.getElementById('title-bar').style.display = 'none';
            document.getElementById('messages').style.display = 'none';
            document.getElementById('input-bar').style.display = 'none';
            document.getElementById('status-bar').style.display = 'none';
            document.getElementById('image-preview').style.display = 'none';
            var overlay = document.getElementById('chat-overlay');
            if (overlay) overlay.style.display = 'none';

            var container = document.createElement('div');
            container.id = 'fullscreen-iframe';
            container.style.cssText = 'position:fixed;top:0;left:0;right:0;bottom:0;z-index:1000;background:#0a0a0a;';

            var iframe = document.createElement('iframe');
            iframe.srcdoc = atob(base64);
            iframe.style.cssText = 'width:100%;height:100%;border:none;';
            iframe.setAttribute('sandbox', 'allow-scripts allow-same-origin');
            var _iframeLoadFired = false;
            iframe.onload = function() {
                if (!_iframeLoadFired) {
                    _iframeLoadFired = true;
                    window.webkit.messageHandlers.chatInput.postMessage('__iframe_loaded__');
                }
            };
            container.appendChild(iframe);
            // Re-fire panel focus when clicking on the fullscreen game
            container.addEventListener('mousedown', function() {
                window.webkit.messageHandlers.chatInput.postMessage('__focus__');
            });
            document.body.appendChild(container);

            // Fallback: if onload doesn't fire within 1s, signal loaded anyway
            setTimeout(function() {
                if (!_iframeLoadFired) {
                    _iframeLoadFired = true;
                    window.webkit.messageHandlers.chatInput.postMessage('__iframe_loaded__');
                }
            }, 1000);

            // Forward keyboard events from parent into the iframe content
            _iframeKeyForwarder = function(e) {
                if (iframe.contentDocument) {
                    iframe.contentDocument.dispatchEvent(new KeyboardEvent(e.type, {
                        key: e.key,
                        code: e.code,
                        keyCode: e.keyCode,
                        which: e.which,
                        bubbles: true,
                        cancelable: true
                    }));
                    e.preventDefault();
                }
            };
            document.addEventListener('keydown', _iframeKeyForwarder);
            document.addEventListener('keyup', _iframeKeyForwarder);

            setTimeout(function() { iframe.focus(); }, 150);
        }

        function showFullscreenIframeUrl(url) {
            document.getElementById('title-bar').style.display = 'none';
            document.getElementById('messages').style.display = 'none';
            document.getElementById('input-bar').style.display = 'none';
            document.getElementById('status-bar').style.display = 'none';
            document.getElementById('image-preview').style.display = 'none';
            var overlay = document.getElementById('chat-overlay');
            if (overlay) overlay.style.display = 'none';

            var container = document.createElement('div');
            container.id = 'fullscreen-iframe';
            container.style.cssText = 'position:fixed;top:0;left:0;right:0;bottom:0;z-index:1000;background:#0a0a0a;';

            var iframe = document.createElement('iframe');
            iframe.src = url;
            iframe.style.cssText = 'width:100%;height:100%;border:none;';
            iframe.setAttribute('sandbox', 'allow-scripts allow-same-origin allow-popups allow-forms');
            container.appendChild(iframe);
            document.body.appendChild(container);

            setTimeout(function() { iframe.focus(); }, 150);
        }

        function hideFullscreenIframe() {
            if (_iframeKeyForwarder) {
                document.removeEventListener('keydown', _iframeKeyForwarder);
                document.removeEventListener('keyup', _iframeKeyForwarder);
                _iframeKeyForwarder = null;
            }

            var container = document.getElementById('fullscreen-iframe');
            if (container) container.remove();

            document.getElementById('title-bar').style.display = '';
            document.getElementById('messages').style.display = '';
            document.getElementById('input-bar').style.display = '';
            document.getElementById('status-bar').style.display = '';
            document.getElementById('image-preview').style.display = '';
            var overlay = document.getElementById('chat-overlay');
            if (overlay) overlay.style.display = '';
        }

        function closePanel() {
            window.webkit.messageHandlers.chatInput.postMessage('__close_panel__');
        }

        setTimeout(() => chatInput.focus(), 200);
        </script>
        </body>
        </html>
        """
    }
}
