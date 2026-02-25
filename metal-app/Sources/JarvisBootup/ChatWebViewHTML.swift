import Foundation

extension ChatWebView {
    static func buildHTML(title: String) -> String {
        let escapedTitle = title.replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "\"", with: "\\\"")
        return """
        <!DOCTYPE html>
        <html>
        <head>
        <meta charset="utf-8">
        <script src="https://cdn.jsdelivr.net/npm/d3@7/dist/d3.min.js"></script>
        <script src="https://cdn.jsdelivr.net/npm/marked@15/marked.min.js"></script>
        <style>
            * { margin: 0; padding: 0; box-sizing: border-box; }
            body {
                background: rgba(0, 0, 0, 0.93);
                color: #00d4ff;
                font-family: Menlo, Monaco, 'Courier New', monospace;
                font-size: 13px;
                line-height: 1.6;
                display: flex;
                flex-direction: column;
                height: 100vh;
                overflow: hidden;
                border: 1px solid rgba(0, 212, 255, 0.08);
                transition: border-color 0.2s ease;
            }
            body.focused {
                border-color: rgba(0, 212, 255, 0.5);
                box-shadow: inset 0 0 12px rgba(0, 212, 255, 0.08);
            }
            #title-bar {
                padding: 14px 20px 8px;
                font-size: 15px;
                font-weight: bold;
                color: #00d4ff;
                text-shadow: 0 0 8px rgba(0, 212, 255, 0.35);
                border-bottom: 1px solid rgba(0, 212, 255, 0.12);
                flex-shrink: 0;
            }
            #messages {
                flex: 1;
                overflow-y: auto;
                padding: 10px 20px;
            }
            #messages::-webkit-scrollbar { width: 3px; }
            #messages::-webkit-scrollbar-track { background: transparent; }
            #messages::-webkit-scrollbar-thumb { background: rgba(0, 212, 255, 0.15); border-radius: 2px; }

            .msg { margin-bottom: 6px; word-wrap: break-word; }
            .msg.gemini { color: #f0ece4; }
            .msg.gemini h1, .msg.gemini h2, .msg.gemini h3 {
                font-size: 14px; margin: 10px 0 4px; color: #f0ece4;
                text-shadow: 0 0 6px rgba(240,236,228,0.15);
            }
            .msg.gemini h1 { font-size: 15px; }
            .msg.gemini p { margin: 4px 0; }
            .msg.gemini ul, .msg.gemini ol { margin: 4px 0 4px 20px; }
            .msg.gemini li { margin: 2px 0; }
            .msg.gemini strong { color: #faf6ee; }
            .msg.gemini code {
                background: rgba(0,212,255,0.08); padding: 1px 4px; border-radius: 2px;
                font-size: 12px;
            }
            .msg.gemini pre {
                background: rgba(0,212,255,0.05); padding: 8px; border-radius: 3px;
                margin: 6px 0; overflow-x: auto;
            }
            .msg.gemini pre code { background: none; padding: 0; }
            .msg.user {
                color: rgba(140, 190, 220, 0.65);
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
                border: 1px solid rgba(0, 212, 255, 0.15);
                display: block;
            }
            .msg.iframe-container {
                margin: 8px 0;
                padding: 0;
            }
            .msg.iframe-container iframe {
                width: 100%;
                border: 1px solid rgba(0, 212, 255, 0.15);
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
            .msg.tool_read  { color: rgba(100, 180, 255, 0.9); border-color: rgba(100, 180, 255, 0.4); }
            .msg.tool_edit  { color: rgba(255, 180, 80, 0.9);  border-color: rgba(255, 180, 80, 0.4); }
            .msg.tool_write { color: rgba(255, 180, 80, 0.9);  border-color: rgba(255, 180, 80, 0.4); }
            .msg.tool_run   { color: rgba(80, 220, 120, 0.9);  border-color: rgba(80, 220, 120, 0.4); }
            .msg.tool_search{ color: rgba(200, 150, 255, 0.9); border-color: rgba(200, 150, 255, 0.4); }
            .msg.tool_list  { color: rgba(0, 212, 255, 0.85);  border-color: rgba(0, 212, 255, 0.35); }
            .msg.tool_data  { color: rgba(0, 212, 200, 0.85);  border-color: rgba(0, 212, 200, 0.35); }
            .msg.tool_tool  { color: rgba(255, 200, 50, 0.85); border-color: rgba(255, 200, 50, 0.35); }
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
                background: rgba(0, 212, 255, 0.03);
                border: 1px solid rgba(0, 212, 255, 0.1);
                border-radius: 4px;
            }
            .chart-container svg text { fill: #00d4ff; font-family: Menlo, monospace; font-size: 11px; }
            .chart-container svg .bar { fill: rgba(0, 212, 255, 0.55); }
            .chart-container svg .bar:hover { fill: rgba(0, 212, 255, 0.8); }
            .chart-container svg .line-path { fill: none; stroke: #00d4ff; stroke-width: 2; }
            .chart-container svg .dot { fill: #00d4ff; }
            .chart-container svg .axis line, .chart-container svg .axis path { stroke: rgba(0, 212, 255, 0.2); }
            .chart-title { font-size: 13px; font-weight: bold; margin-bottom: 6px; color: #00d4ff; }

            #input-bar {
                display: flex;
                padding: 6px 20px 10px;
                border-top: 1px solid rgba(0, 212, 255, 0.12);
                flex-shrink: 0;
            }
            #input-bar textarea {
                flex: 1;
                background: rgba(0, 212, 255, 0.05);
                border: 1px solid rgba(0, 212, 255, 0.15);
                border-radius: 3px;
                color: #00d4ff;
                font-family: Menlo, Monaco, monospace;
                font-size: 13px;
                padding: 7px 10px;
                outline: none;
                resize: none;
                overflow: hidden;
                min-height: 32px;
                max-height: 300px;
                line-height: 1.4;
            }
            #input-bar textarea::placeholder { color: rgba(0, 212, 255, 0.2); }
            #input-bar textarea:focus { border-color: rgba(0, 212, 255, 0.4); }
            #status-bar {
                padding: 4px 20px 0;
                font-size: 10px;
                color: rgba(0, 212, 255, 0.35);
                flex-shrink: 0;
                font-family: Menlo, Monaco, monospace;
            }
            #image-preview {
                display: none;
                padding: 8px 20px;
                border-top: 1px solid rgba(0, 212, 255, 0.12);
                flex-shrink: 0;
                position: relative;
            }
            #image-preview img {
                max-height: 200px;
                max-width: 100%;
                border-radius: 4px;
                border: 1px solid rgba(0, 212, 255, 0.2);
                display: block;
            }
            #image-preview .close-btn {
                position: absolute;
                top: 4px;
                right: 24px;
                color: rgba(0, 212, 255, 0.5);
                cursor: pointer;
                font-size: 16px;
                line-height: 1;
            }
            #image-preview .close-btn:hover {
                color: rgba(0, 212, 255, 0.9);
            }
            #chat-overlay {
                position: fixed;
                top: 8px;
                right: 8px;
                max-width: 280px;
                font-size: 11px;
                line-height: 1.4;
                color: rgba(0, 212, 255, 0.4);
                font-family: Menlo, Monaco, monospace;
                white-space: pre-wrap;
                pointer-events: none;
                z-index: 100;
                text-align: right;
            }
        </style>
        </head>
        <body>
            <div id="chat-overlay"></div>
            <div id="title-bar">[ \(escapedTitle) ]</div>
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
                    lastEscapeJS = 0;
                    window.webkit.messageHandlers.chatInput.postMessage('__escape__');
                } else {
                    lastEscapeJS = now;
                    chatInput.value = '';
                    if (typeof autoGrow === 'function') autoGrow();
                    if (typeof clearImagePreview === 'function') clearImagePreview();
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

        setTimeout(() => chatInput.focus(), 200);
        </script>
        </body>
        </html>
        """
    }
}
