import WebKit

extension ChatWebView {
    func injectAdBlocker(panel idx: Int) {
        guard idx >= 0, idx < panels.count else { return }
        panels[idx].evaluateJavaScript("""
            (function() {
                // Hide common ad selectors
                var css = document.createElement('style');
                css.textContent = `
                    iframe[src*="ad"], iframe[src*="doubleclick"], iframe[src*="googlesyndication"],
                    iframe[src*="adservice"], iframe[id*="ad"], iframe[class*="ad"],
                    div[id*="ad-"], div[class*="ad-container"], div[class*="ad_"],
                    div[class*="ads-"], div[class*="adsbygoogle"], div[id*="google_ads"],
                    ins.adsbygoogle, div[data-ad], div[class*="sponsor"],
                    div[class*="banner"], div[id*="banner"],
                    .sidebar-right, .game-sidebar, .right-sidebar,
                    div[class*="sidebar"] { display: none !important; }
                    /* Force game canvas to fill screen */
                    canvas, .game-container, .game-area, #game, #gameContainer,
                    .game-canvas { position: fixed !important; top: 0 !important;
                    left: 0 !important; width: 100vw !important; height: 100vh !important;
                    z-index: 9999 !important; }
                    body { overflow: hidden !important; }
                `;
                document.head.appendChild(css);

                // Remove ad iframes
                document.querySelectorAll('iframe').forEach(function(f) {
                    var src = (f.src || '').toLowerCase();
                    if (src.includes('ad') || src.includes('doubleclick') || src.includes('googlesyndication')
                        || src.includes('sponsor') || f.offsetWidth < 5) {
                        f.remove();
                    }
                });

                // Remove common ad containers
                var adSelectors = [
                    '[id*="google_ads"]', '.adsbygoogle', '[data-ad-slot]',
                    '[class*="ad-wrapper"]', '[class*="ad_wrapper"]',
                    '[id*="ad_"]', '[id*="ad-"]'
                ];
                adSelectors.forEach(function(sel) {
                    document.querySelectorAll(sel).forEach(function(el) { el.remove(); });
                });
            })();
        """, completionHandler: nil)
    }
}
