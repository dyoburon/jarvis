/**
 * Port of ChatWebViewHTML.swift buildHTML()
 * Adapted for React Native WebView:
 * - window.webkit.messageHandlers → window.ReactNativeWebView.postMessage
 * - Added viewport meta for mobile
 * - Touch-friendly input sizing
 */

export function buildChatHTML(title: string = 'jarvis'): string {
  const escapedTitle = title.replace(/\\/g, '\\\\').replace(/"/g, '\\"');

  return `<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no">
<script src="https://cdn.jsdelivr.net/npm/d3@7/dist/d3.min.js"></script>
<script src="https://cdn.jsdelivr.net/npm/marked@15/marked.min.js"></script>
<style>
    :root {
        --color-panel-bg: rgba(0, 0, 0, 0.93);
        --color-primary: rgba(0, 212, 255, 0.75);
        --color-text: rgba(0, 212, 255, 0.65);
        --color-border: rgba(0, 212, 255, 0.08);
        --color-border-focused: rgba(0, 212, 255, 0.5);
        --color-user-text: rgba(0, 212, 255, 0.85);
        --font-family: Menlo, Monaco, 'Courier New', monospace;
        --font-size: 13px;
        --line-height: 1.5;
        --font-title-size: 12px;
        --color-tool-read: #6cb6ff;
        --color-tool-edit: #e8c44a;
        --color-tool-write: #7ee87e;
        --color-tool-run: #ff6b6b;
        --color-tool-search: #c49bff;
    }
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
        height: 100dvh;
        overflow: hidden;
        -webkit-text-size-adjust: none;
    }
    #title-bar {
        padding: 12px 16px 8px;
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
    #messages {
        flex: 1;
        overflow-y: auto;
        padding: 10px 16px;
        -webkit-overflow-scrolling: touch;
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

    /* Tool activity */
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

    .chart-container {
        margin: 10px 0;
        padding: 14px;
        background: color-mix(in srgb, var(--color-primary) 3%, transparent);
        border: 1px solid color-mix(in srgb, var(--color-primary) 10%, transparent);
        border-radius: 4px;
    }
    .chart-container svg text { fill: var(--color-primary); font-family: var(--font-family); font-size: 11px; }
    .chart-container svg .bar { fill: color-mix(in srgb, var(--color-primary) 55%, transparent); }
    .chart-container svg .line-path { fill: none; stroke: var(--color-primary); stroke-width: 2; }
    .chart-container svg .dot { fill: var(--color-primary); }
    .chart-container svg .axis line, .chart-container svg .axis path { stroke: color-mix(in srgb, var(--color-primary) 20%, transparent); }
    .chart-title { font-size: 13px; font-weight: bold; margin-bottom: 6px; color: var(--color-primary); }

    #input-bar {
        display: flex;
        padding: 8px 16px 12px;
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
        font-size: 16px; /* prevent iOS zoom */
        padding: 10px 12px;
        outline: none;
        resize: none;
        overflow: hidden;
        min-height: 40px;
        max-height: 200px;
        line-height: 1.4;
        -webkit-appearance: none;
    }
    #input-bar textarea::placeholder { color: color-mix(in srgb, var(--color-primary) 20%, transparent); }
    #input-bar textarea:focus { border-color: color-mix(in srgb, var(--color-primary) 40%, transparent); }
    #status-bar {
        padding: 4px 16px 8px;
        font-size: 10px;
        color: color-mix(in srgb, var(--color-primary) 35%, transparent);
        flex-shrink: 0;
        font-family: var(--font-family);
    }
</style>
</head>
<body>
    <div id="title-bar"><span>[ ${escapedTitle} ]</span></div>
    <div id="messages"></div>
    <div id="input-bar">
        <textarea id="chat-input" rows="1" placeholder="Type a question..." autocomplete="off"></textarea>
    </div>
    <div id="status-bar"></div>
<script>
    marked.setOptions({ breaks: true, gfm: true });

    const messages = document.getElementById('messages');
    const chatInput = document.getElementById('chat-input');
    let currentSpeaker = null;
    let currentEl = null;
    let geminiBuffer = '';
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

        if (speaker.startsWith('tool_') && speaker !== 'tool_result') {
            currentSpeaker = null;
            currentEl = null;
            geminiBuffer = '';
            const el = document.createElement('div');
            el.className = 'msg tool-activity ' + speaker;
            el.textContent = text;
            messages.appendChild(el);
            scrollIfNear();
            return;
        }

        if (speaker === 'tool_result') {
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

        // Gemini: accumulate and re-render as markdown
        geminiBuffer += text;

        if (speaker !== currentSpeaker || !currentEl) {
            currentEl = document.createElement('div');
            currentEl.className = 'msg gemini';
            messages.appendChild(currentEl);
            currentSpeaker = speaker;
        }

        // Split buffer into text segments and chart blocks
        const parts = geminiBuffer.split(/(\\x60\\x60\\x60chart\\n[\\s\\S]*?\\n\\x60\\x60\\x60)/g);
        currentEl.innerHTML = '';

        for (const part of parts) {
            const chartMatch = part.match(/^\\x60\\x60\\x60chart\\n([\\s\\S]*?)\\n\\x60\\x60\\x60$/);
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
                const span = document.createElement('span');
                span.innerHTML = marked.parse(part);
                currentEl.appendChild(span);
            }
        }
        scrollIfNear();
    }

    function buildChart(container, config) {
        const w = Math.min(window.innerWidth - 64, 400);
        const h = 180;
        const m = {top: 15, right: 15, bottom: 35, left: 45};
        const iw = w - m.left - m.right, ih = h - m.top - m.bottom;

        const svg = d3.select(container).append('svg')
            .attr('width', w).attr('height', h)
            .append('g').attr('transform', 'translate(' + m.left + ',' + m.top + ')');

        const labels = config.labels || [];
        const values = config.values || [];

        if (config.type === 'bar') {
            const x = d3.scaleBand().domain(labels).range([0, iw]).padding(0.3);
            const y = d3.scaleLinear().domain([0, d3.max(values) * 1.1]).range([ih, 0]);
            svg.append('g').attr('class','axis').attr('transform','translate(0,' + ih + ')').call(d3.axisBottom(x));
            svg.append('g').attr('class','axis').call(d3.axisLeft(y).ticks(5));
            svg.selectAll('.bar').data(values).join('rect')
                .attr('class','bar').attr('x',function(d,i){return x(labels[i])}).attr('y',function(d){return y(d)})
                .attr('width',x.bandwidth()).attr('height',function(d){return ih-y(d)});
        } else if (config.type === 'line') {
            const x = d3.scalePoint().domain(labels).range([0, iw]);
            const y = d3.scaleLinear().domain([0, d3.max(values) * 1.1]).range([ih, 0]);
            svg.append('g').attr('class','axis').attr('transform','translate(0,' + ih + ')').call(d3.axisBottom(x));
            svg.append('g').attr('class','axis').call(d3.axisLeft(y).ticks(5));
            const line = d3.line().x(function(d,i){return x(labels[i])}).y(function(d){return y(d)});
            svg.append('path').datum(values).attr('class','line-path').attr('d',line);
            svg.selectAll('.dot').data(values).join('circle')
                .attr('class','dot').attr('cx',function(d,i){return x(labels[i])}).attr('cy',function(d){return y(d)}).attr('r',3);
        }
    }

    function autoGrow() {
        chatInput.style.height = 'auto';
        chatInput.style.overflow = 'hidden';
        const sh = chatInput.scrollHeight;
        chatInput.style.height = Math.min(sh, 200) + 'px';
        chatInput.style.overflow = sh > 200 ? 'auto' : 'hidden';
    }

    chatInput.addEventListener('input', autoGrow);

    chatInput.addEventListener('keydown', function(e) {
        if (e.key === 'Enter' && !e.shiftKey && chatInput.value.trim()) {
            e.preventDefault();
            var text = chatInput.value.trim();
            window.ReactNativeWebView.postMessage(JSON.stringify({ type: 'chat', text: text }));
            chatInput.value = '';
            autoGrow();
        }
    });

    function setStatus(text) {
        document.getElementById('status-bar').textContent = text;
    }

    // Listen for messages from React Native
    window.addEventListener('message', function(e) {
        try {
            var data = JSON.parse(e.data);
            if (data.type === 'append') {
                appendChunk(data.speaker, data.text);
            } else if (data.type === 'status') {
                setStatus(data.text);
            }
        } catch(err) {}
    });

    // Signal ready
    window.ReactNativeWebView.postMessage(JSON.stringify({ type: 'ready' }));
</script>
</body>
</html>`;
}
